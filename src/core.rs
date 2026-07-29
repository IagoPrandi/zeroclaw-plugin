use std::collections::{HashMap, HashSet};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

use crate::{
    config::GuardianConfig,
    decoders::{DecodeSummary, decode_transaction},
    error::GuardianError,
    input::{GuardianInput, OutputLanguage, TransactionSource},
    limits::Budget,
    output::{
        Coverage, Decision, ExecutionReport, ExecutionStatus, FeeReport, GuardianReport,
        InstructionCoverage, Participants, SourceReport,
    },
    risk::{RiskContext, evaluate},
    rpc::{RpcClient, RpcTransport},
    simulation::{
        ExecutionEffects, confirmed_effects, has_durable_nonce, priority_fee_lamports,
        simulation_effects,
    },
    state::{StateEvidence, load_required_state},
    transaction::{
        DecodedLookupTable, NormalizedTransaction, decode_lookup_table, normalize_base64,
    },
};

/// Validate the stable contract and produce the conservative foundation report.
///
/// This entry point is progressively populated by the parsing, RPC, decoder,
/// simulation, and risk milestones. It is already fail-closed.
///
/// # Errors
///
/// Returns a typed, safe error for invalid input or configuration.
#[allow(clippy::implicit_hasher)]
#[allow(clippy::too_many_lines)]
pub fn analyze_contract(
    input_json: &str,
    config_values: &HashMap<String, String>,
) -> Result<GuardianReport, GuardianError> {
    let input: GuardianInput = serde_json::from_str(input_json)
        .map_err(|error| GuardianError::invalid_input(format!("invalid arguments: {error}")))?;
    input.validate()?;
    let config = GuardianConfig::parse(config_values)?;
    if !config.allowed_clusters.contains(input.cluster.as_str()) {
        return Err(GuardianError::invalid_input(
            "the requested cluster is not allowed by operator policy",
        ));
    }

    let mut hasher = Sha256::new();
    let canonical_input = serde_json::to_vec(
        &serde_json::from_str::<serde_json::Value>(input_json)
            .map_err(|_| GuardianError::invalid_input("invalid JSON input"))?,
    )
    .map_err(|_| GuardianError::Internal)?;
    hasher.update(canonical_input);
    hasher.update(config.policy_version.as_bytes());
    let analysis_id = format!("sha256:{}", hex::encode(hasher.finalize()));
    let (source_type, signature, normalized) = match &input.source {
        TransactionSource::Serialized { transaction_base64 } => {
            if transaction_base64.len() > 8192 {
                return Err(GuardianError::TransactionTooLarge);
            }
            let normalized = normalize_base64(transaction_base64, &config.limits, &HashMap::new())?;
            ("serialized", None, Some(normalized))
        }
        TransactionSource::Confirmed { signature } => {
            if !(80..=100).contains(&signature.len())
                || bs58::decode(signature)
                    .into_vec()
                    .map_or(true, |bytes| bytes.len() != 64)
            {
                return Err(GuardianError::invalid_input(
                    "signature must be a base58-encoded 64-byte Solana signature",
                ));
            }
            ("confirmed", Some(signature.clone()), None)
        }
    };
    let mut participants = Participants::default();
    let decoded = normalized.as_ref().map(decode_transaction).transpose()?;
    let (transaction_version, fee_payer, instruction_total, lookups_resolved) =
        if let Some(transaction) = &normalized {
            participants.signers = transaction
                .account_keys
                .iter()
                .filter(|key| key.signer)
                .map(|key| key.address)
                .collect();
            participants.writable_accounts = transaction
                .account_keys
                .iter()
                .filter(|key| key.writable)
                .map(|key| key.address)
                .collect();
            let programs: HashSet<_> = transaction
                .instructions
                .iter()
                .map(|instruction| instruction.program_id)
                .collect();
            participants.programs = programs.into_iter().collect();
            participants.programs.sort_unstable();
            if let Some(decoded) = &decoded {
                participants
                    .unknown_programs
                    .clone_from(&decoded.unknown_programs);
            }
            (
                transaction.version.as_str().to_owned(),
                Some(transaction.fee_payer),
                transaction.instructions.len(),
                true,
            )
        } else {
            ("rpc_pending".to_owned(), None, 0, true)
        };
    let empty_decoded = DecodeSummary::default();
    let empty_state = StateEvidence::default();
    let risk = evaluate(&RiskContext {
        source: &input.source,
        observed_wallets: &input.observed_wallets,
        expected_intent: input.expected_intent.as_ref(),
        config: &config,
        decoded: decoded.as_ref().unwrap_or(&empty_decoded),
        asset_deltas: &[],
        execution_status: ExecutionStatus::NotSimulated,
        execution_units: None,
        simulation_available: false,
        inner_instructions_available: false,
        token_2022_extensions: &[],
        state: &empty_state,
        fee_payer,
        total_fee_lamports: None,
        state_balances_are_post_execution: false,
    });
    let summary = match input.output_language {
        crate::input::OutputLanguage::English => match risk.decision {
            Decision::Allow => "No blocking rule was identified.".to_owned(),
            Decision::Review => "Human review is required before trust or signing.".to_owned(),
            Decision::Block => "Do not trust or sign this transaction.".to_owned(),
        },
        crate::input::OutputLanguage::PortugueseBrazil => match risk.decision {
            Decision::Allow => "Nenhuma regra de bloqueio foi identificada.".to_owned(),
            Decision::Review => {
                "Revisão humana é necessária antes de confiar ou assinar.".to_owned()
            }
            Decision::Block => "Não confie nem assine esta transação.".to_owned(),
        },
    };

    Ok(GuardianReport {
        schema_version: "1.0.0",
        plugin_version: env!("CARGO_PKG_VERSION"),
        policy_version: config.policy_version,
        analysis_id,
        source: SourceReport {
            source_type,
            cluster: input.cluster.as_str().to_owned(),
            signature,
            slot: None,
            transaction_version,
        },
        decision: risk.decision,
        risk_level: risk.risk_level,
        risk_score: risk.risk_score,
        confidence: risk.confidence,
        analysis_complete: risk.analysis_complete,
        execution: ExecutionReport {
            status: ExecutionStatus::NotSimulated,
            error: None,
            units_consumed: None,
            logs_truncated: false,
            return_data: None,
        },
        fees: FeeReport {
            fee_payer,
            ..FeeReport::default()
        },
        participants,
        actions: decoded
            .as_ref()
            .map(|summary| {
                summary
                    .actions
                    .clone()
                    .into_iter()
                    .map(crate::decoders::ActionData::into_output)
                    .collect()
            })
            .unwrap_or_default(),
        asset_deltas: Vec::new(),
        authority_changes: Vec::new(),
        findings: risk.findings,
        coverage: Coverage {
            top_level_instructions: InstructionCoverage {
                decoded: decoded.as_ref().map_or(0, |summary| summary.decoded),
                total: instruction_total,
            },
            inner_instructions_available: false,
            address_lookup_tables_resolved: lookups_resolved,
            simulation_available: false,
            unresolved_components: decoded.as_ref().map_or_else(
                || vec!["confirmed transaction RPC fetch is pending".to_owned()],
                |summary| {
                    summary
                        .unknown_programs
                        .iter()
                        .map(|program| format!("unknown program {program}"))
                        .collect()
                },
            ),
        },
        summary,
        limitations: vec![
            "Candidate simulation is not connected in this execution path.".to_owned(),
            "Simulation describes recent state and cannot guarantee future execution.".to_owned(),
        ],
        generated_at: generated_at()?,
    })
}

/// Run the complete RPC-backed analysis path.
///
/// # Errors
///
/// Returns typed input/config/RPC/parsing errors. Expected execution failures
/// remain successful reports with a block or review decision.
#[allow(clippy::implicit_hasher)]
#[allow(clippy::too_many_lines)]
pub fn analyze_with_rpc<T: RpcTransport>(
    input_json: &str,
    config_values: &HashMap<String, String>,
    transport: T,
) -> Result<GuardianReport, GuardianError> {
    let input: GuardianInput = serde_json::from_str(input_json)
        .map_err(|error| GuardianError::invalid_input(format!("invalid arguments: {error}")))?;
    input.validate()?;
    let config = GuardianConfig::parse(config_values)?;
    if !config.allowed_clusters.contains(input.cluster.as_str()) {
        return Err(GuardianError::invalid_input(
            "the requested cluster is not allowed by operator policy",
        ));
    }
    let endpoint = config
        .rpc_endpoints
        .get(input.cluster.as_str())
        .cloned()
        .ok_or_else(|| GuardianError::invalid_config("cluster has no RPC endpoint"))?;
    let mut rpc = RpcClient::new(
        endpoint,
        config.request_timeout_ms,
        transport,
        Budget::new(config.limits.clone()),
    );

    match &input.source {
        TransactionSource::Serialized { transaction_base64 } => {
            let lookup_tables = load_lookup_tables(transaction_base64, &config, &mut rpc)?;
            let normalized = normalize_base64(transaction_base64, &config.limits, &lookup_tables)?;
            let decoded = decode_transaction(&normalized)?;
            let durable_nonce = has_durable_nonce(&decoded.actions);
            let mut report = analyze_contract(input_json, config_values)?;
            if !config.enable_simulation {
                return Ok(report);
            }
            let simulation =
                match rpc.simulate_transaction(transaction_base64, "processed", !durable_nonce) {
                    Ok(simulation) => simulation,
                    Err(error) => {
                        report.execution.status = ExecutionStatus::RpcUnavailable;
                        report.execution.error = Some(format!(
                            "Solana RPC simulation is unavailable ({}).",
                            error.code()
                        ));
                        report.limitations.push(format!(
                            "Candidate simulation could not be obtained ({}).",
                            error.code()
                        ));
                        return Ok(report);
                    }
                };
            let inner_available = simulation
                .pointer("/value/innerInstructions")
                .is_some_and(|value| !value.is_null());
            let effects = simulation_effects(&simulation, &normalized)?;
            let state_actions = combined_actions(&decoded, &effects);
            let state = load_required_state(
                &mut rpc,
                &state_actions,
                &input.observed_wallets,
                None,
                false,
            )?;
            let (total_fee, fee_limitation) =
                match candidate_total_fee(transaction_base64, &mut rpc) {
                    Ok(fee) => (Some(fee), None),
                    Err(error) => (
                        None,
                        Some(format!(
                            "Base fee estimation could not be obtained ({}).",
                            error.code()
                        )),
                    ),
                };
            let mut report = finalize_report(
                report,
                &input,
                &config,
                &normalized,
                &decoded,
                effects,
                true,
                inner_available,
                total_fee,
                state,
            );
            if let Some(limitation) = fee_limitation {
                report.limitations.push(limitation);
            }
            Ok(report)
        }
        TransactionSource::Confirmed { signature } => {
            let result = rpc.get_transaction(signature, "confirmed")?;
            let encoded = result
                .pointer("/transaction/0")
                .and_then(serde_json::Value::as_str)
                .ok_or(GuardianError::RpcProtocol)?
                .to_owned();
            let lookup_tables = load_lookup_tables(&encoded, &config, &mut rpc)?;
            let normalized = normalize_base64(&encoded, &config.limits, &lookup_tables)?;
            let decoded = decode_transaction(&normalized)?;
            let meta = result.get("meta").ok_or(GuardianError::RpcProtocol)?;
            let effects = confirmed_effects(meta, &normalized)?;
            let slot = result.get("slot").and_then(serde_json::Value::as_u64);
            let state_actions = combined_actions(&decoded, &effects);
            let state = load_required_state(
                &mut rpc,
                &state_actions,
                &input.observed_wallets,
                slot,
                true,
            )?;
            let mut synthetic: serde_json::Value =
                serde_json::from_str(input_json).map_err(|_| GuardianError::InvalidInput {
                    message: "invalid arguments".to_owned(),
                })?;
            synthetic["source"] = serde_json::json!({
                "type": "serialized",
                "transaction_base64": encoded
            });
            let mut report = analyze_contract(&synthetic.to_string(), config_values)?;
            report.source.source_type = "confirmed";
            report.source.signature = Some(signature.clone());
            report.source.slot = slot;
            let inner_available = meta
                .get("innerInstructions")
                .is_some_and(|value| !value.is_null());
            let total_fee = effects.fee_lamports;
            Ok(finalize_report(
                report,
                &input,
                &config,
                &normalized,
                &decoded,
                effects,
                true,
                inner_available,
                total_fee,
                state,
            ))
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn finalize_report(
    mut report: GuardianReport,
    input: &GuardianInput,
    config: &GuardianConfig,
    normalized: &NormalizedTransaction,
    decoded: &DecodeSummary,
    effects: ExecutionEffects,
    simulation_available: bool,
    inner_available: bool,
    total_fee: Option<u64>,
    mut state: StateEvidence,
) -> GuardianReport {
    let mut risk_decoded = decoded.clone();
    for inner in &effects.inner_actions {
        if inner.action.known {
            risk_decoded.decoded = risk_decoded.decoded.saturating_add(1);
        } else if !risk_decoded
            .unknown_programs
            .contains(&inner.action.program_id)
        {
            risk_decoded.unknown_programs.push(inner.action.program_id);
        }
        risk_decoded.actions.push(inner.action.clone());
    }
    risk_decoded.unknown_programs.sort_unstable();
    if matches!(input.source, TransactionSource::Confirmed { .. }) {
        state.sol_balances = effects
            .post_sol_balances
            .iter()
            .copied()
            .filter(|(account, _)| input.observed_wallets.contains(account))
            .collect();
    }
    let risk = evaluate(&RiskContext {
        source: &input.source,
        observed_wallets: &input.observed_wallets,
        expected_intent: input.expected_intent.as_ref(),
        config,
        decoded: &risk_decoded,
        asset_deltas: &effects.asset_deltas,
        execution_status: effects.status,
        execution_units: effects.units_consumed,
        simulation_available,
        inner_instructions_available: inner_available,
        token_2022_extensions: &state.token_2022_extensions,
        state: &state,
        fee_payer: Some(normalized.fee_payer),
        total_fee_lamports: total_fee,
        state_balances_are_post_execution: matches!(
            input.source,
            TransactionSource::Confirmed { .. }
        ),
    });
    report.decision = risk.decision;
    report.risk_level = risk.risk_level;
    report.risk_score = risk.risk_score;
    report.confidence = risk.confidence;
    report.analysis_complete = risk.analysis_complete;
    report.findings = risk.findings;
    report.execution = ExecutionReport {
        status: effects.status,
        error: effects.error,
        units_consumed: effects.units_consumed,
        logs_truncated: effects.logs_truncated,
        return_data: effects.return_data,
    };
    report.actions = decoded
        .actions
        .clone()
        .into_iter()
        .map(crate::decoders::ActionData::into_output)
        .collect();
    report
        .actions
        .extend(effects.inner_actions.into_iter().map(|inner| {
            let mut action = inner.action.into_output();
            action.inner_index = Some(inner.inner_index);
            action.instruction_index = inner.top_level_index;
            action.details.insert(
                "stack_height".to_owned(),
                serde_json::json!(inner.stack_height),
            );
            action
        }));
    report.actions.sort_by_key(|action| {
        (
            action.instruction_index,
            action
                .inner_index
                .map_or(0, |index| index.saturating_add(1)),
        )
    });
    report.authority_changes = authority_changes(&report.actions);
    report.asset_deltas = effects
        .asset_deltas
        .into_iter()
        .map(crate::simulation::AssetDeltaData::into_output)
        .collect();
    let compute_limit = action_decimal(&report, "set_compute_unit_limit", "units");
    let compute_price = action_decimal(&report, "set_compute_unit_price", "micro_lamports");
    report.fees.compute_unit_limit = compute_limit.map(|value| value.to_string());
    report.fees.compute_unit_price_micro_lamports = compute_price.map(|value| value.to_string());
    let priority_fee = compute_limit
        .zip(compute_price)
        .and_then(|(limit, price)| priority_fee_lamports(limit, price).ok());
    let base_fee = total_fee.and_then(|total| total.checked_sub(priority_fee.unwrap_or(0)));
    report.fees.base_fee_lamports = base_fee.map(|value| value.to_string());
    report.fees.priority_fee_lamports = priority_fee.map(|value| value.to_string());
    report.fees.total_estimated_fee_lamports = total_fee.map(|value| value.to_string());
    report.coverage.inner_instructions_available = inner_available;
    report.coverage.simulation_available = simulation_available;
    report.coverage.top_level_instructions.decoded = decoded.decoded;
    report.coverage.top_level_instructions.total = normalized.instructions.len();
    report.coverage.unresolved_components = risk_decoded
        .unknown_programs
        .iter()
        .map(|program| format!("unknown program {program}"))
        .collect();
    report.summary = decision_summary(risk.decision, input.output_language);
    report.limitations = if matches!(input.source, TransactionSource::Serialized { .. }) {
        vec!["Simulation reflects recent state and does not guarantee future execution.".to_owned()]
    } else {
        Vec::new()
    };
    report.limitations.extend(
        risk_decoded
            .unknown_programs
            .iter()
            .map(|program| format!("No decoder is available for program {program}.")),
    );
    report
}

fn combined_actions(
    decoded: &DecodeSummary,
    effects: &ExecutionEffects,
) -> Vec<crate::decoders::ActionData> {
    decoded
        .actions
        .iter()
        .cloned()
        .chain(
            effects
                .inner_actions
                .iter()
                .map(|inner| inner.action.clone()),
        )
        .collect()
}

fn authority_changes(actions: &[crate::output::Action]) -> Vec<crate::output::AuthorityChange> {
    let mut changes = Vec::new();
    for action in actions {
        let authority_type = match action.kind.as_str() {
            "set_authority" => action
                .details
                .get("authority_type")
                .and_then(serde_json::Value::as_u64)
                .map_or_else(
                    || "token_authority".to_owned(),
                    |value| format!("token_{value}"),
                ),
            "set_authority_checked" => "program_upgrade_authority".to_owned(),
            "authorize_nonce_account" => "nonce_authority".to_owned(),
            _ => continue,
        };
        let new_authority = action
            .details
            .get("new_authority")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse().ok())
            .or_else(|| {
                (action.kind == "set_authority_checked")
                    .then(|| action.accounts.last().copied())
                    .flatten()
            });
        if let Some(account) = action.accounts.first().copied() {
            changes.push(crate::output::AuthorityChange {
                instruction_index: action.instruction_index,
                account,
                authority_type,
                old_authority: None,
                new_authority,
            });
        }
    }
    changes.sort_by_key(|change| (change.instruction_index, change.account));
    changes
}

fn candidate_total_fee<T: RpcTransport>(
    encoded_transaction: &str,
    rpc: &mut RpcClient<T>,
) -> Result<u64, GuardianError> {
    let wire = STANDARD
        .decode(encoded_transaction)
        .map_err(|_| GuardianError::Base64Decode)?;
    let transaction: solana_transaction::versioned::VersionedTransaction =
        bincode::deserialize(&wire).map_err(|_| GuardianError::TransactionDeserialize)?;
    let message = bincode::serialize(&transaction.message).map_err(|_| GuardianError::Internal)?;
    let result = rpc.get_fee_for_message(&STANDARD.encode(message), "confirmed")?;
    result
        .get("value")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| result.as_u64())
        .ok_or(GuardianError::RpcProtocol)
}

fn load_lookup_tables<T: RpcTransport>(
    encoded_transaction: &str,
    config: &GuardianConfig,
    rpc: &mut RpcClient<T>,
) -> Result<HashMap<crate::address::Address32, DecodedLookupTable>, GuardianError> {
    let wire = STANDARD
        .decode(encoded_transaction)
        .map_err(|_| GuardianError::Base64Decode)?;
    let transaction: solana_transaction::versioned::VersionedTransaction =
        bincode::deserialize(&wire).map_err(|_| GuardianError::TransactionDeserialize)?;
    let Some(lookups) = transaction.message.address_table_lookups() else {
        return Ok(HashMap::new());
    };
    if lookups.is_empty() {
        return Ok(HashMap::new());
    }
    let mut addresses: Vec<_> = lookups
        .iter()
        .map(|lookup| lookup.account_key.to_string())
        .collect();
    addresses.sort();
    addresses.dedup();
    let result = rpc.get_multiple_accounts(&addresses, "confirmed", None)?;
    let accounts = result
        .get("value")
        .and_then(serde_json::Value::as_array)
        .ok_or(GuardianError::RpcProtocol)?;
    if accounts.len() != addresses.len() {
        return Err(GuardianError::RpcProtocol);
    }
    let alt_owner: crate::address::Address32 = "AddressLookupTab1e1111111111111111111111111"
        .parse()
        .map_err(|_| GuardianError::Internal)?;
    let mut output = HashMap::new();
    for (address, account) in addresses.iter().zip(accounts) {
        if account.is_null() {
            return Err(GuardianError::AddressLookupTable);
        }
        let owner: crate::address::Address32 = account
            .get("owner")
            .and_then(serde_json::Value::as_str)
            .ok_or(GuardianError::RpcProtocol)?
            .parse()
            .map_err(|_| GuardianError::RpcProtocol)?;
        if owner != alt_owner {
            return Err(GuardianError::AddressLookupTable);
        }
        let encoded_data = account
            .pointer("/data/0")
            .or_else(|| account.get("data"))
            .and_then(serde_json::Value::as_str)
            .ok_or(GuardianError::RpcProtocol)?;
        let data = STANDARD
            .decode(encoded_data)
            .map_err(|_| GuardianError::RpcProtocol)?;
        let table = decode_lookup_table(&data)?;
        let table_address = address.parse().map_err(|_| GuardianError::RpcProtocol)?;
        if output.insert(table_address, table).is_some() {
            return Err(GuardianError::RpcProtocol);
        }
    }
    if output.len() > config.limits.max_accounts {
        return Err(GuardianError::AddressLookupTable);
    }
    Ok(output)
}

fn generated_at() -> Result<String, GuardianError> {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| GuardianError::Internal)
}

fn action_decimal(report: &GuardianReport, kind: &str, field: &str) -> Option<u64> {
    report
        .actions
        .iter()
        .find(|action| action.kind == kind)
        .and_then(|action| action.details.get(field))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse().ok())
}

fn decision_summary(decision: Decision, language: OutputLanguage) -> String {
    match (decision, language) {
        (Decision::Allow, OutputLanguage::English) => "No blocking rule was identified.".to_owned(),
        (Decision::Review, OutputLanguage::English) => {
            "Human review is required before trust or signing.".to_owned()
        }
        (Decision::Block, OutputLanguage::English) => {
            "Do not trust or sign this transaction.".to_owned()
        }
        (Decision::Allow, OutputLanguage::PortugueseBrazil) => {
            "Nenhuma regra de bloqueio foi identificada.".to_owned()
        }
        (Decision::Review, OutputLanguage::PortugueseBrazil) => {
            "Revisão humana é necessária antes de confiar ou assinar.".to_owned()
        }
        (Decision::Block, OutputLanguage::PortugueseBrazil) => {
            "Não confie nem assine esta transação.".to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, VecDeque};

    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use solana_message::{Message, MessageHeader, VersionedMessage};
    use solana_transaction::versioned::VersionedTransaction;

    use crate::{
        config::valid_test_config,
        rpc::{HttpResponse, RpcTransport, TransportError},
    };

    struct MockTransport {
        responses: VecDeque<HttpResponse>,
    }

    impl RpcTransport for MockTransport {
        fn post(
            &mut self,
            _endpoint: &str,
            _body: &[u8],
            _timeout_ms: u64,
            _max_response_bytes: usize,
        ) -> Result<HttpResponse, TransportError> {
            self.responses.pop_front().ok_or(TransportError::Other)
        }
    }

    fn valid_encoded_transaction() -> String {
        let transaction = VersionedTransaction {
            signatures: vec![solana_transaction::Signature::default()],
            message: VersionedMessage::Legacy(Message {
                header: MessageHeader {
                    num_required_signatures: 1,
                    num_readonly_signed_accounts: 0,
                    num_readonly_unsigned_accounts: 0,
                },
                account_keys: vec![solana_message::Address::new_from_array([1; 32])],
                recent_blockhash: solana_message::Hash::default(),
                instructions: vec![],
            }),
        };
        STANDARD.encode(bincode::serialize(&transaction).unwrap_or_default())
    }

    #[test]
    fn golden_empty_contract_is_fail_closed() {
        let input = serde_json::json!({
            "source":{"type":"serialized","transaction_base64":valid_encoded_transaction()},
            "cluster":"devnet"
        })
        .to_string();
        let report = super::analyze_contract(&input, &valid_test_config());
        assert!(report.is_ok());
        let value = serde_json::to_value(report.unwrap_or_else(|_| unreachable!()));
        assert_eq!(
            value.as_ref().ok().and_then(|v| v.get("decision")),
            Some(&serde_json::json!("block"))
        );
        assert_eq!(
            value.as_ref().ok().and_then(|v| v.get("analysis_complete")),
            Some(&serde_json::json!(false))
        );
    }

    #[test]
    fn unknown_input_field_is_controlled_error() {
        let input = r#"{
            "source":{"type":"serialized","transaction_base64":"AQ=="},
            "cluster":"devnet",
            "endpoint":"https://attacker.invalid"
        }"#;
        assert!(super::analyze_contract(input, &valid_test_config()).is_err());
    }

    #[test]
    fn rpc_backed_candidate_can_complete_and_allow() {
        let input = serde_json::json!({
            "source":{"type":"serialized","transaction_base64":valid_encoded_transaction()},
            "cluster":"devnet"
        })
        .to_string();
        let transport = MockTransport {
            responses: [
                r#"{"jsonrpc":"2.0","id":1,"result":{"value":{"err":null,"logs":[],"unitsConsumed":0,"innerInstructions":[]}}}"#,
                r#"{"jsonrpc":"2.0","id":2,"result":{"context":{"slot":1},"value":5000}}"#,
            ]
            .into_iter()
            .map(|body| HttpResponse {
                status: 200,
                body: body.as_bytes().to_vec(),
            })
            .collect(),
        };
        let report = super::analyze_with_rpc(&input, &valid_test_config(), transport);
        assert!(matches!(
            report.map(|value| (value.decision, value.analysis_complete)),
            Ok((crate::output::Decision::Allow, true))
        ));
    }

    #[test]
    fn authority_change_output_is_derived_from_canonical_actions() {
        let account = crate::address::Address32::new([1; 32]);
        let new_authority = crate::address::Address32::new([2; 32]);
        let actions = [crate::output::Action {
            instruction_index: 3,
            inner_index: None,
            kind: "set_authority".to_owned(),
            program_id: crate::address::Address32::new([9; 32]),
            accounts: vec![account],
            details: BTreeMap::from([
                ("authority_type".to_owned(), serde_json::json!(0)),
                (
                    "new_authority".to_owned(),
                    serde_json::json!(new_authority.to_string()),
                ),
            ]),
        }];

        let changes = super::authority_changes(&actions);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].account, account);
        assert_eq!(changes[0].new_authority, Some(new_authority));
    }
}
