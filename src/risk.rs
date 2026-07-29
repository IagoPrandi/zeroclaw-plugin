use serde_json::{Value, json};

use crate::{
    address::Address32,
    config::{GuardianConfig, PolicyEffect},
    decoders::{ActionData, DecodeSummary},
    input::{ExpectedIntent, TransactionSource},
    output::{Decision, DecisionEffect, ExecutionStatus, Finding, Severity},
    simulation::{AssetDeltaData, priority_fee_lamports},
    state::StateEvidence,
};

#[derive(Debug)]
pub struct RiskContext<'a> {
    pub source: &'a TransactionSource,
    pub observed_wallets: &'a [Address32],
    pub expected_intent: Option<&'a ExpectedIntent>,
    pub config: &'a GuardianConfig,
    pub decoded: &'a DecodeSummary,
    pub asset_deltas: &'a [AssetDeltaData],
    pub execution_status: ExecutionStatus,
    pub execution_units: Option<u64>,
    pub simulation_available: bool,
    pub inner_instructions_available: bool,
    pub token_2022_extensions: &'a [String],
    pub state: &'a StateEvidence,
    pub fee_payer: Option<Address32>,
    pub total_fee_lamports: Option<u64>,
    pub state_balances_are_post_execution: bool,
}

#[derive(Debug)]
pub struct RiskResult {
    pub findings: Vec<Finding>,
    pub decision: Decision,
    pub risk_level: Severity,
    pub risk_score: u8,
    pub confidence: f64,
    pub analysis_complete: bool,
}

/// Evaluate deterministic rules and reduce them to one canonical decision.
///
/// The operator policy is always evaluated before intent restrictions; intent
/// can add restrictions but cannot relax operator caps.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn evaluate(context: &RiskContext<'_>) -> RiskResult {
    let mut findings = Vec::new();
    evaluate_coverage(context, &mut findings);
    evaluate_execution(context, &mut findings);
    evaluate_programs(context, &mut findings);
    evaluate_transfers(context, &mut findings);
    evaluate_authority(context, &mut findings);
    evaluate_token_2022(context, &mut findings);
    evaluate_fees(context, &mut findings);
    evaluate_intent(context, &mut findings);
    canonical_sort(&mut findings);

    let analysis_complete = context.decoded.decoded == context.decoded.actions.len()
        && (!matches!(context.source, TransactionSource::Serialized { .. })
            || context.simulation_available);
    let confidence = confidence(context);
    let decision = if findings
        .iter()
        .any(|finding| finding.decision_effect == DecisionEffect::Block)
        || (!analysis_complete && context.config.fail_closed)
    {
        Decision::Block
    } else if findings
        .iter()
        .any(|finding| finding.decision_effect == DecisionEffect::Review)
        || confidence < 0.8
    {
        Decision::Review
    } else {
        Decision::Allow
    };
    let risk_score = findings
        .iter()
        .map(|finding| severity_score(finding.severity))
        .fold(0_u16, u16::saturating_add)
        .min(100) as u8;
    let risk_level = findings
        .iter()
        .map(|finding| finding.severity)
        .max()
        .unwrap_or(Severity::Low);
    RiskResult {
        findings,
        decision,
        risk_level,
        risk_score,
        confidence,
        analysis_complete,
    }
}

fn evaluate_coverage(context: &RiskContext<'_>, output: &mut Vec<Finding>) {
    for issue in &context.state.critical_issues {
        push(
            output,
            "COV-006",
            Severity::Critical,
            "coverage",
            "Critical account state is unavailable or malformed",
            DecisionEffect::Block,
            [
                ("account", json!(issue.account.to_string())),
                ("reason", json!(issue.reason)),
            ],
        );
    }
    if context.state.rpc_inconsistent {
        push(
            output,
            "COV-007",
            Severity::Critical,
            "coverage",
            "Solana RPC returned inconsistent account state",
            DecisionEffect::Block,
            [],
        );
    }
    for program in &context.decoded.unknown_programs {
        let observed_writable = context.decoded.actions.iter().any(|action| {
            !action.known
                && action.program_id == *program
                && action
                    .accounts
                    .iter()
                    .any(|account| context.observed_wallets.contains(account))
        });
        push(
            output,
            "COV-003",
            Severity::High,
            "coverage",
            if observed_writable {
                "Unknown program touches an observed account"
            } else {
                "Unknown program lacks a decoder"
            },
            effect(context.config.policy.unknown_program_policy),
            [
                ("program_id", json!(program.to_string())),
                ("observed_account", json!(observed_writable)),
            ],
        );
    }
    if matches!(context.source, TransactionSource::Serialized { .. })
        && !context.simulation_available
    {
        push(
            output,
            "COV-004",
            Severity::High,
            "coverage",
            "Simulation is unavailable for a candidate transaction",
            effect(context.config.policy.simulation_unavailable_policy),
            [],
        );
    }
    if !context.inner_instructions_available && context.decoded.actions.len() > 1 {
        push(
            output,
            "COV-005",
            Severity::Medium,
            "coverage",
            "Inner instructions are unavailable for a complex transaction",
            DecisionEffect::Review,
            [],
        );
    }
}

fn evaluate_execution(context: &RiskContext<'_>, output: &mut Vec<Finding>) {
    match context.execution_status {
        ExecutionStatus::SimulationFailed => push(
            output,
            "EXEC-001",
            Severity::High,
            "execution",
            "Transaction simulation failed",
            DecisionEffect::Block,
            [],
        ),
        ExecutionStatus::ConfirmedFailed => push(
            output,
            "EXEC-002",
            Severity::Medium,
            "execution",
            "Confirmed transaction failed",
            DecisionEffect::Review,
            [],
        ),
        _ => {}
    }
    if context.decoded.duplicate_compute_budget {
        push(
            output,
            "EXEC-003",
            Severity::High,
            "execution",
            "Compute budget instruction is duplicated",
            DecisionEffect::Block,
            [],
        );
    }
    let requested = compute_limit(&context.decoded.actions);
    if let (Some(consumed), Some(limit)) = (context.execution_units, requested)
        && consumed.saturating_mul(100) >= limit.saturating_mul(90)
    {
        push(
            output,
            "EXEC-004",
            Severity::Medium,
            "execution",
            "Compute consumption is close to the requested limit",
            DecisionEffect::Review,
            [("consumed", json!(consumed)), ("limit", json!(limit))],
        );
    }
}

fn evaluate_programs(context: &RiskContext<'_>, output: &mut Vec<Finding>) {
    for action in &context.decoded.actions {
        if context
            .config
            .policy
            .blocked_programs
            .contains(&action.program_id)
        {
            push(
                output,
                "PROG-001",
                Severity::Critical,
                "program",
                "Program is blocked by operator policy",
                DecisionEffect::Block,
                evidence(action),
            );
        }
        if !context.config.policy.allowed_programs.is_empty()
            && !context
                .config
                .policy
                .allowed_programs
                .contains(&action.program_id)
        {
            push(
                output,
                "PROG-002",
                Severity::High,
                "program",
                "Program is outside the strict operator allowlist",
                DecisionEffect::Block,
                evidence(action),
            );
        }
        let (rule, title) = match action.kind.as_str() {
            "upgrade" | "deploy_with_max_data_len" | "extend_program" => {
                ("PROG-003", "Upgradeable program mutation")
            }
            "set_authority_checked" => ("PROG-004", "Program upgrade authority change"),
            "close" => ("PROG-005", "Program or program-data close"),
            _ => continue,
        };
        push(
            output,
            rule,
            Severity::Critical,
            "program",
            title,
            DecisionEffect::Block,
            evidence(action),
        );
    }
    for unknown in &context.decoded.unknown_programs {
        if context
            .asset_deltas
            .iter()
            .any(|delta| context.observed_wallets.contains(&delta.account) && delta.raw_delta < 0)
        {
            push(
                output,
                "PROG-006",
                Severity::Critical,
                "program",
                "Unknown program has high observed impact",
                DecisionEffect::Block,
                [("program_id", json!(unknown.to_string()))],
            );
        }
    }
}

#[allow(clippy::too_many_lines)]
fn evaluate_transfers(context: &RiskContext<'_>, output: &mut Vec<Finding>) {
    let sol_out = effective_sol_out(context);
    if sol_out > u128::from(context.config.policy.sol_out_block_lamports) {
        push(
            output,
            "XFER-001",
            Severity::Critical,
            "transfer",
            "SOL outflow exceeds the operator hard cap",
            DecisionEffect::Block,
            [("lamports", json!(sol_out.to_string()))],
        );
    }
    for action in transfer_actions(&context.decoded.actions) {
        if let Some(recipient) = recipient(action) {
            if context
                .config
                .policy
                .blocked_recipients
                .contains(&recipient)
            {
                push(
                    output,
                    "XFER-003",
                    Severity::Critical,
                    "transfer",
                    "Recipient is blocked by operator policy",
                    DecisionEffect::Block,
                    evidence(action),
                );
            } else if !context
                .config
                .policy
                .known_recipients
                .contains_key(&recipient)
                && sol_out > u128::from(context.config.policy.sol_out_review_lamports)
            {
                push(
                    output,
                    "XFER-004",
                    Severity::Medium,
                    "transfer",
                    "Large transfer uses an unknown recipient",
                    DecisionEffect::Review,
                    evidence(action),
                );
            }
        }
    }
    for delta in context.asset_deltas {
        if delta.mint.is_some()
            && context.observed_wallets.contains(&delta.account)
            && delta.raw_delta.unsigned_abs() > u128::from(u64::MAX)
        {
            push(
                output,
                "XFER-002",
                Severity::Critical,
                "transfer",
                "Token outflow exceeds the supported hard-cap range",
                DecisionEffect::Block,
                [("account", json!(delta.account.to_string()))],
            );
        }
    }
    if let Some(intent) = context.expected_intent {
        if sol_out > 0 && intent.max_sol_out_lamports.is_none() {
            push(
                output,
                "XFER-005",
                Severity::Critical,
                "transfer",
                "SOL outflow was not declared by structured intent",
                DecisionEffect::Block,
                [("lamports", json!(sol_out.to_string()))],
            );
        }
        let mut undeclared_mints: Vec<_> = context
            .asset_deltas
            .iter()
            .filter(|delta| {
                delta.raw_delta < 0 && context.observed_wallets.contains(&delta.account)
            })
            .filter_map(|delta| delta.mint)
            .filter(|mint| !intent.token_limits.iter().any(|limit| limit.mint == *mint))
            .collect();
        undeclared_mints.sort_unstable();
        undeclared_mints.dedup();
        for mint in undeclared_mints {
            push(
                output,
                "XFER-005",
                Severity::Critical,
                "transfer",
                "Token outflow was not declared by structured intent",
                DecisionEffect::Block,
                [("mint", json!(mint.to_string()))],
            );
        }
        for limit in &intent.token_limits {
            let incoming = observed_in(context.asset_deltas, context.observed_wallets, limit.mint);
            if limit
                .min_in_raw
                .as_ref()
                .and_then(|value| value.parse::<u128>().ok())
                .is_some_and(|minimum| incoming < minimum)
            {
                push(
                    output,
                    "XFER-006",
                    Severity::Critical,
                    "transfer",
                    "Token input is below the declared minimum",
                    DecisionEffect::Block,
                    [
                        ("mint", json!(limit.mint.to_string())),
                        ("incoming_raw", json!(incoming.to_string())),
                    ],
                );
            }
        }
    }
    for (wallet, balance) in &context.state.sol_balances {
        let post_balance = if context.state_balances_are_post_execution {
            u128::from(*balance)
        } else {
            let out = sol_out_for_wallet(context, *wallet);
            let fee = if context.fee_payer == Some(*wallet) {
                u128::from(context.total_fee_lamports.unwrap_or(0))
            } else {
                0
            };
            u128::from(*balance).saturating_sub(out.saturating_add(fee))
        };
        if post_balance < u128::from(context.config.policy.minimum_sol_reserve_lamports) {
            push(
                output,
                "XFER-007",
                Severity::Medium,
                "transfer",
                "Observed wallet falls below the operator minimum SOL reserve",
                DecisionEffect::Review,
                [
                    ("wallet", json!(wallet.to_string())),
                    ("post_lamports", json!(post_balance.to_string())),
                ],
            );
        }
    }
}

#[allow(clippy::too_many_lines)]
fn evaluate_authority(context: &RiskContext<'_>, output: &mut Vec<Finding>) {
    for action in &context.decoded.actions {
        if action.program_id.to_string() == "BPFLoaderUpgradeab1e11111111111111111111111"
            && matches!(
                action.kind.as_str(),
                "set_authority" | "set_authority_checked"
            )
        {
            push(
                output,
                "AUTH-008",
                Severity::Critical,
                "authority_or_account",
                "Upgradeable program authority changes",
                DecisionEffect::Block,
                evidence(action),
            );
            continue;
        }
        let (rule, severity, title, effect_value) = match action.kind.as_str() {
            "set_authority" => (
                match action.details.get("authority_type").and_then(Value::as_u64) {
                    Some(0) => "AUTH-002",
                    Some(1) => "AUTH-003",
                    _ => "AUTH-001",
                },
                Severity::Critical,
                "Token authority changes",
                DecisionEffect::Block,
            ),
            "approve" | "approve_checked" => (
                "AUTH-004",
                Severity::Critical,
                "Unplanned token delegate approval",
                DecisionEffect::Block,
            ),
            "authorize_nonce_account" => (
                "AUTH-007",
                Severity::High,
                "Nonce authority changes",
                DecisionEffect::Review,
            ),
            "close_account" => (
                "ACCT-001",
                Severity::Critical,
                "Observed token account may close",
                DecisionEffect::Block,
            ),
            "freeze_account" => (
                "ACCT-003",
                Severity::Critical,
                "Token account may freeze",
                DecisionEffect::Block,
            ),
            "burn" | "burn_checked" => (
                "ACCT-004",
                Severity::High,
                "Tokens may be burned",
                DecisionEffect::Block,
            ),
            "mint_to" | "mint_to_checked" => (
                "ACCT-005",
                Severity::High,
                "Tokens may be minted",
                DecisionEffect::Review,
            ),
            _ => continue,
        };
        if action.accounts.is_empty()
            || action
                .accounts
                .iter()
                .any(|account| context.observed_wallets.contains(account))
        {
            push(
                output,
                rule,
                severity,
                "authority_or_account",
                title,
                effect_value,
                evidence(action),
            );
        }
        if matches!(action.kind.as_str(), "approve" | "approve_checked")
            && action
                .details
                .get("amount_raw")
                .and_then(Value::as_str)
                .and_then(|value| value.parse::<u128>().ok())
                .is_some_and(|amount| amount > 0)
        {
            push(
                output,
                "AUTH-005",
                Severity::Critical,
                "authority_or_account",
                "Token delegate receives nonzero spending capacity",
                DecisionEffect::Block,
                evidence(action),
            );
        }
        if action.kind == "close_account"
            && action
                .accounts
                .first()
                .is_some_and(|account| context.observed_wallets.contains(account))
            && action.accounts.get(1).is_some_and(|destination| {
                !context
                    .config
                    .policy
                    .known_recipients
                    .contains_key(destination)
                    && !context
                        .expected_intent
                        .is_some_and(|intent| intent.allowed_recipients.contains(destination))
            })
        {
            push(
                output,
                "ACCT-002",
                Severity::Critical,
                "authority_or_account",
                "Token account rent may be returned to an unexpected destination",
                DecisionEffect::Block,
                evidence(action),
            );
        }
    }
    for mismatch in &context.state.owner_mismatches {
        push(
            output,
            "ACCT-006",
            Severity::Critical,
            "authority_or_account",
            "Account owner is incompatible with the instruction decoder",
            DecisionEffect::Block,
            [
                ("account", json!(mismatch.account.to_string())),
                ("expected_owner", json!(mismatch.expected_owner.to_string())),
                ("actual_owner", json!(mismatch.actual_owner.to_string())),
            ],
        );
    }
}

fn evaluate_token_2022(context: &RiskContext<'_>, output: &mut Vec<Finding>) {
    for extension in context.token_2022_extensions {
        let (rule, severity, title, effect_value) = match extension.as_str() {
            "transfer_hook" => (
                "T22-001",
                Severity::High,
                "Token-2022 transfer hook is enabled",
                effect(context.config.policy.token2022_transfer_hook_policy),
            ),
            "transfer_fee_config" => (
                "T22-002",
                Severity::Medium,
                "Token-2022 transfer fee is configured",
                DecisionEffect::Review,
            ),
            "confidential_transfer_mint"
            | "confidential_transfer_account"
            | "confidential_transfer_fee_config"
            | "confidential_transfer_fee_amount" => (
                "T22-003",
                Severity::High,
                "Confidential Token-2022 effect cannot be fully interpreted",
                DecisionEffect::Review,
            ),
            "default_account_state" => (
                "T22-004",
                Severity::Medium,
                "Token-2022 default account state requires review",
                DecisionEffect::Review,
            ),
            "non_transferable" | "non_transferable_account" => (
                "T22-005",
                Severity::High,
                "Token-2022 asset is non-transferable",
                DecisionEffect::Block,
            ),
            "cpi_guard" => (
                "T22-006",
                Severity::Medium,
                "Token-2022 CPI guard is present",
                DecisionEffect::Review,
            ),
            "permanent_delegate" => (
                "AUTH-006",
                Severity::Critical,
                "Token-2022 permanent delegate is configured",
                effect(context.config.policy.token2022_permanent_delegate_policy),
            ),
            _ => continue,
        };
        push(
            output,
            rule,
            severity,
            "token_2022",
            title,
            effect_value,
            [("extension", json!(extension))],
        );
    }
}

fn evaluate_fees(context: &RiskContext<'_>, output: &mut Vec<Finding>) {
    if !context.observed_wallets.is_empty()
        && context
            .fee_payer
            .is_some_and(|fee_payer| !context.observed_wallets.contains(&fee_payer))
    {
        push(
            output,
            "FEE-003",
            Severity::Medium,
            "fee",
            "Fee payer is outside the observed wallet set",
            DecisionEffect::Review,
            [(
                "fee_payer",
                json!(context.fee_payer.map(|value| value.to_string())),
            )],
        );
    }
    let Some(limit) = compute_limit(&context.decoded.actions) else {
        return;
    };
    let Some(price) = compute_price(&context.decoded.actions) else {
        return;
    };
    let Ok(fee) = priority_fee_lamports(limit, price) else {
        push(
            output,
            "FEE-002",
            Severity::Critical,
            "fee",
            "Priority fee arithmetic overflow",
            DecisionEffect::Block,
            [],
        );
        return;
    };
    if fee > context.config.policy.priority_fee_block_lamports {
        push(
            output,
            "FEE-002",
            Severity::Critical,
            "fee",
            "Priority fee exceeds the operator hard cap",
            DecisionEffect::Block,
            [("lamports", json!(fee.to_string()))],
        );
    } else if fee > context.config.policy.priority_fee_review_lamports {
        push(
            output,
            "FEE-001",
            Severity::Medium,
            "fee",
            "Priority fee exceeds the review threshold",
            DecisionEffect::Review,
            [("lamports", json!(fee.to_string()))],
        );
    }
    if let Some(consumed) = context.execution_units
        && limit > consumed.saturating_mul(4)
    {
        push(
            output,
            "FEE-004",
            Severity::Medium,
            "fee",
            "Requested compute limit greatly exceeds consumption",
            DecisionEffect::Review,
            [("limit", json!(limit)), ("consumed", json!(consumed))],
        );
    }
}

#[allow(clippy::too_many_lines)]
fn evaluate_intent(context: &RiskContext<'_>, output: &mut Vec<Finding>) {
    let Some(intent) = context.expected_intent else {
        return;
    };
    if !intent.allowed_programs.is_empty() {
        for action in &context.decoded.actions {
            if !intent.allowed_programs.contains(&action.program_id) {
                push(
                    output,
                    "INT-001",
                    Severity::High,
                    "intent",
                    "Observed program was not declared in the expected intent",
                    DecisionEffect::Review,
                    evidence(action),
                );
            }
        }
    }
    if !intent.allowed_recipients.is_empty() {
        for action in transfer_actions(&context.decoded.actions) {
            if let Some(recipient) = recipient(action)
                && !intent.allowed_recipients.contains(&recipient)
            {
                push(
                    output,
                    "INT-002",
                    Severity::Critical,
                    "intent",
                    "Observed recipient was not declared in the expected intent",
                    DecisionEffect::Block,
                    evidence(action),
                );
            }
        }
    }
    if let Some(maximum) = intent
        .max_sol_out_lamports
        .as_ref()
        .and_then(|value| value.parse::<u128>().ok())
        && effective_sol_out(context) > maximum
    {
        push(
            output,
            "INT-005",
            Severity::Critical,
            "intent",
            "Observed SOL outflow exceeds the declared intent",
            DecisionEffect::Block,
            [("maximum", json!(maximum.to_string()))],
        );
    }
    for limit in &intent.token_limits {
        let out = observed_out(
            context.asset_deltas,
            context.observed_wallets,
            Some(limit.mint),
        );
        if limit
            .max_out_raw
            .as_ref()
            .and_then(|value| value.parse::<u128>().ok())
            .is_some_and(|maximum| out > maximum)
        {
            push(
                output,
                "INT-005",
                Severity::Critical,
                "intent",
                "Observed token outflow exceeds the declared intent",
                DecisionEffect::Block,
                [("mint", json!(limit.mint.to_string()))],
            );
        }
        let incoming = observed_in(context.asset_deltas, context.observed_wallets, limit.mint);
        if limit
            .min_in_raw
            .as_ref()
            .and_then(|value| value.parse::<u128>().ok())
            .is_some_and(|minimum| incoming < minimum)
        {
            push(
                output,
                "INT-006",
                Severity::Critical,
                "intent",
                "Observed token input is below the declared minimum",
                DecisionEffect::Block,
                [("mint", json!(limit.mint.to_string()))],
            );
        }
    }
    if !intent.token_limits.is_empty() {
        let mut unexpected_mints: Vec<_> = context
            .asset_deltas
            .iter()
            .filter(|delta| {
                context.observed_wallets.contains(&delta.account) && delta.raw_delta != 0
            })
            .filter_map(|delta| delta.mint)
            .filter(|mint| !intent.token_limits.iter().any(|limit| limit.mint == *mint))
            .collect();
        unexpected_mints.sort_unstable();
        unexpected_mints.dedup();
        for mint in unexpected_mints {
            push(
                output,
                "INT-004",
                Severity::Critical,
                "intent",
                "Observed token asset differs from structured intent",
                DecisionEffect::Block,
                [("mint", json!(mint.to_string()))],
            );
        }
    }
    if context.decoded.actions.iter().any(|action| {
        matches!(
            action.kind.as_str(),
            "set_authority" | "approve" | "approve_checked"
        )
    }) {
        push(
            output,
            "INT-003",
            Severity::Critical,
            "intent",
            "Authority action was not explicitly represented by structured intent",
            DecisionEffect::Block,
            [],
        );
    }
}

fn push<const N: usize>(
    output: &mut Vec<Finding>,
    rule_id: &str,
    severity: Severity,
    category: &str,
    title: &str,
    decision_effect: DecisionEffect,
    evidence_values: [(&str, Value); N],
) {
    output.push(Finding {
        rule_id: rule_id.to_owned(),
        severity,
        category: category.to_owned(),
        title: title.to_owned(),
        explanation: title.to_owned(),
        evidence: evidence_values
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
        decision_effect,
    });
}

fn evidence(action: &ActionData) -> [(&'static str, Value); 2] {
    [
        ("instruction_index", json!(action.instruction_index)),
        ("program_id", json!(action.program_id.to_string())),
    ]
}

fn effect(value: PolicyEffect) -> DecisionEffect {
    match value {
        PolicyEffect::None => DecisionEffect::None,
        PolicyEffect::Review => DecisionEffect::Review,
        PolicyEffect::Block => DecisionEffect::Block,
    }
}

fn transfer_actions(actions: &[ActionData]) -> impl Iterator<Item = &ActionData> {
    actions.iter().filter(|action| {
        matches!(
            action.kind.as_str(),
            "transfer" | "transfer_with_seed" | "transfer_checked"
        )
    })
}

fn recipient(action: &ActionData) -> Option<Address32> {
    let index = if matches!(
        action.kind.as_str(),
        "transfer_checked" | "transfer_with_seed"
    ) {
        2
    } else {
        1
    };
    action.accounts.get(index).copied()
}

fn effective_sol_out(context: &RiskContext<'_>) -> u128 {
    observed_out(context.asset_deltas, context.observed_wallets, None).max(static_sol_out(
        &context.decoded.actions,
        context.observed_wallets,
    ))
}

fn static_sol_out(actions: &[ActionData], wallets: &[Address32]) -> u128 {
    actions
        .iter()
        .filter(|action| {
            action.program_id.to_string() == "11111111111111111111111111111111"
                && matches!(
                    action.kind.as_str(),
                    "create_account" | "transfer" | "transfer_with_seed" | "withdraw_nonce_account"
                )
                && action
                    .accounts
                    .first()
                    .is_some_and(|source| wallets.contains(source))
        })
        .filter_map(|action| action.details.get("lamports"))
        .filter_map(Value::as_str)
        .filter_map(|amount| amount.parse::<u128>().ok())
        .fold(0_u128, u128::saturating_add)
}

fn sol_out_for_wallet(context: &RiskContext<'_>, wallet: Address32) -> u128 {
    observed_out(context.asset_deltas, &[wallet], None)
        .max(static_sol_out(&context.decoded.actions, &[wallet]))
}

fn observed_out(deltas: &[AssetDeltaData], wallets: &[Address32], mint: Option<Address32>) -> u128 {
    deltas
        .iter()
        .filter(|delta| {
            wallets.contains(&delta.account) && delta.raw_delta < 0 && delta.mint == mint
        })
        .map(|delta| delta.raw_delta.unsigned_abs())
        .fold(0, u128::saturating_add)
}

fn observed_in(deltas: &[AssetDeltaData], wallets: &[Address32], mint: Address32) -> u128 {
    deltas
        .iter()
        .filter(|delta| {
            wallets.contains(&delta.account) && delta.raw_delta > 0 && delta.mint == Some(mint)
        })
        .map(|delta| delta.raw_delta.unsigned_abs())
        .fold(0, u128::saturating_add)
}

fn compute_limit(actions: &[ActionData]) -> Option<u64> {
    actions
        .iter()
        .find(|action| action.kind == "set_compute_unit_limit")
        .and_then(|action| action.details.get("units"))
        .and_then(Value::as_str)
        .and_then(|value| value.parse().ok())
}

fn compute_price(actions: &[ActionData]) -> Option<u64> {
    actions
        .iter()
        .find(|action| action.kind == "set_compute_unit_price")
        .and_then(|action| action.details.get("micro_lamports"))
        .and_then(Value::as_str)
        .and_then(|value| value.parse().ok())
}

fn confidence(context: &RiskContext<'_>) -> f64 {
    let unknown_penalty: f64 = match context.decoded.unknown_programs.len() {
        0 => 0.0,
        1 => 0.15,
        2 => 0.30,
        _ => 0.45,
    };
    let inner_penalty =
        if !context.inner_instructions_available && context.decoded.actions.len() > 1 {
            0.15
        } else {
            0.0
        };
    let simulation_penalty = if matches!(context.source, TransactionSource::Serialized { .. })
        && !context.simulation_available
    {
        0.35
    } else {
        0.0
    };
    (1.0 - unknown_penalty - inner_penalty - simulation_penalty).max(0.0)
}

fn severity_score(severity: Severity) -> u16 {
    match severity {
        Severity::Critical => 40,
        Severity::High => 25,
        Severity::Medium => 10,
        Severity::Low => 3,
    }
}

fn canonical_sort(findings: &mut [Finding]) {
    findings.sort_by(|left, right| {
        effect_rank(left.decision_effect)
            .cmp(&effect_rank(right.decision_effect))
            .then_with(|| severity_rank(left.severity).cmp(&severity_rank(right.severity)))
            .then_with(|| left.rule_id.cmp(&right.rule_id))
    });
}

const fn effect_rank(effect: DecisionEffect) -> u8 {
    match effect {
        DecisionEffect::Block => 0,
        DecisionEffect::Review => 1,
        DecisionEffect::None => 2,
    }
}

const fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{RiskContext, evaluate};
    use crate::{
        address::Address32,
        config::{GuardianConfig, valid_test_config},
        decoders::{ActionData, DecodeSummary},
        input::{ExpectedIntent, TokenLimit, TransactionSource},
        output::{Decision, ExecutionStatus},
        simulation::AssetDeltaData,
        state::{CriticalStateIssue, OwnerMismatch, StateEvidence},
    };

    static EMPTY_STATE: StateEvidence = StateEvidence {
        token_2022_extensions: Vec::new(),
        critical_issues: Vec::new(),
        owner_mismatches: Vec::new(),
        sol_balances: Vec::new(),
        rpc_inconsistent: false,
    };

    fn config() -> GuardianConfig {
        GuardianConfig::parse(&valid_test_config()).unwrap_or_else(|_| unreachable!())
    }

    fn context<'a>(
        source: &'a TransactionSource,
        config: &'a GuardianConfig,
        decoded: &'a DecodeSummary,
        deltas: &'a [AssetDeltaData],
        wallets: &'a [Address32],
    ) -> RiskContext<'a> {
        RiskContext {
            source,
            observed_wallets: wallets,
            expected_intent: None,
            config,
            decoded,
            asset_deltas: deltas,
            execution_status: ExecutionStatus::SimulationSucceeded,
            execution_units: None,
            simulation_available: true,
            inner_instructions_available: true,
            token_2022_extensions: &[],
            state: &EMPTY_STATE,
            fee_payer: None,
            total_fee_lamports: None,
            state_balances_are_post_execution: false,
        }
    }

    #[test]
    fn hard_cap_blocks_regardless_of_other_findings() {
        let config = config();
        let source = TransactionSource::Serialized {
            transaction_base64: "AQ==".to_owned(),
        };
        let wallet = Address32::new([1; 32]);
        let deltas = [AssetDeltaData {
            account: wallet,
            owner: None,
            mint: None,
            program_id: None,
            asset: "SOL".to_owned(),
            raw_delta: -2_000_000_000,
            decimals: Some(9),
        }];
        let decoded = DecodeSummary::default();
        let result = evaluate(&context(&source, &config, &decoded, &deltas, &[wallet]));
        assert_eq!(result.decision, Decision::Block);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.rule_id == "XFER-001")
        );
    }

    #[test]
    fn decoded_system_transfer_enforces_hard_cap_without_balance_deltas() {
        let config = config();
        let source = TransactionSource::Serialized {
            transaction_base64: "AQ==".to_owned(),
        };
        let wallet = Address32::new([1; 32]);
        let recipient = Address32::new([2; 32]);
        let system_program = Address32::new([0; 32]);
        let decoded = DecodeSummary {
            actions: vec![ActionData {
                instruction_index: 0,
                kind: "transfer".to_owned(),
                program_id: system_program,
                accounts: vec![wallet, recipient],
                details: BTreeMap::from([("lamports".to_owned(), serde_json::json!("2000000000"))]),
                known: true,
            }],
            decoded: 1,
            unknown_programs: Vec::new(),
            duplicate_compute_budget: false,
        };

        let result = evaluate(&context(&source, &config, &decoded, &[], &[wallet]));

        assert_eq!(result.decision, Decision::Block);
        assert!(result.findings.iter().any(|finding| {
            finding.rule_id == "XFER-001"
                && finding.evidence.get("lamports") == Some(&serde_json::json!("2000000000"))
        }));
    }

    #[test]
    fn authority_and_unknown_program_are_not_silently_allowed() {
        let config = config();
        let source = TransactionSource::Serialized {
            transaction_base64: "AQ==".to_owned(),
        };
        let wallet = Address32::new([1; 32]);
        let unknown = Address32::new([9; 32]);
        let decoded = DecodeSummary {
            actions: vec![ActionData {
                instruction_index: 0,
                kind: "set_authority".to_owned(),
                program_id: unknown,
                accounts: vec![wallet],
                details: BTreeMap::new(),
                known: false,
            }],
            decoded: 0,
            unknown_programs: vec![unknown],
            duplicate_compute_budget: false,
        };
        let result = evaluate(&context(&source, &config, &decoded, &[], &[wallet]));
        assert_eq!(result.decision, Decision::Block);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.rule_id == "AUTH-001")
        );
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.rule_id == "COV-003")
        );
    }

    #[test]
    fn unknown_program_without_observed_wallet_has_explanatory_finding() {
        let config = config();
        let source = TransactionSource::Confirmed {
            signature: "1".repeat(88),
        };
        let unknown = Address32::new([9; 32]);
        let decoded = DecodeSummary {
            actions: vec![ActionData {
                instruction_index: 0,
                kind: "unknown_program".to_owned(),
                program_id: unknown,
                accounts: Vec::new(),
                details: BTreeMap::new(),
                known: false,
            }],
            decoded: 0,
            unknown_programs: vec![unknown],
            duplicate_compute_budget: false,
        };

        let result = evaluate(&context(&source, &config, &decoded, &[], &[]));

        assert_eq!(result.decision, Decision::Block);
        assert!(result.findings.iter().any(|finding| {
            finding.rule_id == "COV-003"
                && finding.evidence.get("observed_account") == Some(&serde_json::json!(false))
        }));
    }

    #[test]
    fn state_coverage_fee_and_reserve_rules_are_explicit() {
        let config = config();
        let source = TransactionSource::Confirmed {
            signature: "1".repeat(88),
        };
        let wallet = Address32::new([1; 32]);
        let other = Address32::new([2; 32]);
        let expected_owner = Address32::new([3; 32]);
        let actual_owner = Address32::new([4; 32]);
        let decoded = DecodeSummary::default();
        let state = StateEvidence {
            critical_issues: vec![CriticalStateIssue {
                account: wallet,
                reason: "account_not_found",
            }],
            owner_mismatches: vec![OwnerMismatch {
                account: wallet,
                expected_owner,
                actual_owner,
            }],
            sol_balances: vec![(wallet, 1)],
            rpc_inconsistent: true,
            ..StateEvidence::default()
        };
        let wallets = [wallet];
        let mut risk_context = context(&source, &config, &decoded, &[], &wallets);
        risk_context.state = &state;
        risk_context.fee_payer = Some(other);
        risk_context.state_balances_are_post_execution = true;

        let result = evaluate(&risk_context);
        let ids: Vec<_> = result
            .findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect();

        for required in ["COV-006", "COV-007", "ACCT-006", "FEE-003", "XFER-007"] {
            assert!(ids.contains(&required), "missing {required}");
        }
    }

    #[test]
    fn authority_delegate_and_close_rules_cover_critical_actions() {
        let config = config();
        let source = TransactionSource::Serialized {
            transaction_base64: "AQ==".to_owned(),
        };
        let wallet = Address32::new([1; 32]);
        let destination = Address32::new([2; 32]);
        let token_program: Address32 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        let loader: Address32 = "BPFLoaderUpgradeab1e11111111111111111111111"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        let decoded = DecodeSummary {
            actions: vec![
                ActionData {
                    instruction_index: 0,
                    kind: "approve".to_owned(),
                    program_id: token_program,
                    accounts: vec![wallet, destination],
                    details: BTreeMap::from([("amount_raw".to_owned(), serde_json::json!("1"))]),
                    known: true,
                },
                ActionData {
                    instruction_index: 1,
                    kind: "close_account".to_owned(),
                    program_id: token_program,
                    accounts: vec![wallet, destination],
                    details: BTreeMap::new(),
                    known: true,
                },
                ActionData {
                    instruction_index: 2,
                    kind: "set_authority_checked".to_owned(),
                    program_id: loader,
                    accounts: vec![wallet, destination],
                    details: BTreeMap::new(),
                    known: true,
                },
            ],
            decoded: 3,
            unknown_programs: Vec::new(),
            duplicate_compute_budget: false,
        };

        let result = evaluate(&context(&source, &config, &decoded, &[], &[wallet]));
        let ids: Vec<_> = result
            .findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect();

        for required in ["AUTH-005", "AUTH-008", "ACCT-002"] {
            assert!(ids.contains(&required), "missing {required}");
        }
    }

    #[test]
    fn undeclared_assets_and_minimum_input_emit_transfer_and_intent_rules() {
        let config = config();
        let source = TransactionSource::Confirmed {
            signature: "1".repeat(88),
        };
        let wallet = Address32::new([1; 32]);
        let declared_mint = Address32::new([2; 32]);
        let unexpected_mint = Address32::new([3; 32]);
        let decoded = DecodeSummary::default();
        let deltas = [AssetDeltaData {
            account: wallet,
            owner: Some(wallet),
            mint: Some(unexpected_mint),
            program_id: None,
            asset: "token".to_owned(),
            raw_delta: -10,
            decimals: Some(0),
        }];
        let intent = ExpectedIntent {
            description: None,
            allowed_programs: Vec::new(),
            allowed_recipients: Vec::new(),
            max_sol_out_lamports: None,
            token_limits: vec![TokenLimit {
                mint: declared_mint,
                max_out_raw: None,
                min_in_raw: Some("1".to_owned()),
            }],
        };
        let wallets = [wallet];
        let mut risk_context = context(&source, &config, &decoded, &deltas, &wallets);
        risk_context.expected_intent = Some(&intent);

        let result = evaluate(&risk_context);
        let ids: Vec<_> = result
            .findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect();

        for required in ["XFER-005", "XFER-006", "INT-004", "INT-006"] {
            assert!(ids.contains(&required), "missing {required}");
        }
    }

    #[test]
    fn clean_complete_analysis_allows() {
        let config = config();
        let source = TransactionSource::Serialized {
            transaction_base64: "AQ==".to_owned(),
        };
        let decoded = DecodeSummary::default();
        let result = evaluate(&context(&source, &config, &decoded, &[], &[]));
        assert_eq!(result.decision, Decision::Allow);
        assert!((result.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn coverage_execution_program_and_fee_rule_families_have_positive_cases() {
        let mut config = config();
        let source = TransactionSource::Serialized {
            transaction_base64: "AQ==".to_owned(),
        };
        let wallet = Address32::new([1; 32]);
        let other = Address32::new([2; 32]);
        let unknown = Address32::new([9; 32]);
        let loader: Address32 = "BPFLoaderUpgradeab1e11111111111111111111111"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        let system: Address32 = "11111111111111111111111111111111"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        config.policy.blocked_programs.insert(loader);
        config.policy.allowed_programs.insert(system);
        let decoded = DecodeSummary {
            actions: vec![
                ActionData {
                    instruction_index: 0,
                    kind: "unknown_program".to_owned(),
                    program_id: unknown,
                    accounts: vec![wallet],
                    details: BTreeMap::new(),
                    known: false,
                },
                ActionData {
                    instruction_index: 1,
                    kind: "set_compute_unit_limit".to_owned(),
                    program_id: system,
                    accounts: Vec::new(),
                    details: BTreeMap::from([("units".to_owned(), serde_json::json!("100000"))]),
                    known: true,
                },
                ActionData {
                    instruction_index: 2,
                    kind: "set_compute_unit_price".to_owned(),
                    program_id: system,
                    accounts: Vec::new(),
                    details: BTreeMap::from([(
                        "micro_lamports".to_owned(),
                        serde_json::json!("2000000"),
                    )]),
                    known: true,
                },
                ActionData {
                    instruction_index: 3,
                    kind: "upgrade".to_owned(),
                    program_id: loader,
                    accounts: vec![wallet],
                    details: BTreeMap::new(),
                    known: true,
                },
            ],
            decoded: 3,
            unknown_programs: vec![unknown],
            duplicate_compute_budget: true,
        };
        let deltas = [AssetDeltaData {
            account: wallet,
            owner: None,
            mint: None,
            program_id: None,
            asset: "SOL".to_owned(),
            raw_delta: -1,
            decimals: Some(9),
        }];
        let state = StateEvidence {
            critical_issues: vec![CriticalStateIssue {
                account: wallet,
                reason: "account_not_found",
            }],
            rpc_inconsistent: true,
            ..StateEvidence::default()
        };
        let wallets = [wallet];
        let mut risk_context = context(&source, &config, &decoded, &deltas, &wallets);
        risk_context.execution_status = ExecutionStatus::SimulationFailed;
        risk_context.execution_units = Some(95_000);
        risk_context.simulation_available = false;
        risk_context.inner_instructions_available = false;
        risk_context.state = &state;
        risk_context.fee_payer = Some(other);
        let result = evaluate(&risk_context);
        let ids: Vec<_> = result
            .findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect();
        for required in [
            "COV-003", "COV-004", "COV-005", "COV-006", "COV-007", "EXEC-001", "EXEC-003",
            "EXEC-004", "PROG-001", "PROG-002", "PROG-003", "PROG-006", "FEE-001", "FEE-003",
        ] {
            assert!(ids.contains(&required), "missing {required}");
        }

        let confirmed = TransactionSource::Confirmed {
            signature: "1".repeat(88),
        };
        let mut failed = context(&confirmed, &config, &decoded, &[], &[]);
        failed.execution_status = ExecutionStatus::ConfirmedFailed;
        let failed_ids: Vec<_> = evaluate(&failed)
            .findings
            .into_iter()
            .map(|finding| finding.rule_id)
            .collect();
        assert!(failed_ids.iter().any(|rule| rule == "EXEC-002"));

        let high_fee_decoded = DecodeSummary {
            actions: vec![
                ActionData {
                    instruction_index: 0,
                    kind: "set_compute_unit_limit".to_owned(),
                    program_id: system,
                    accounts: Vec::new(),
                    details: BTreeMap::from([("units".to_owned(), serde_json::json!("100000"))]),
                    known: true,
                },
                ActionData {
                    instruction_index: 1,
                    kind: "set_compute_unit_price".to_owned(),
                    program_id: system,
                    accounts: Vec::new(),
                    details: BTreeMap::from([(
                        "micro_lamports".to_owned(),
                        serde_json::json!("20000000"),
                    )]),
                    known: true,
                },
            ],
            decoded: 2,
            unknown_programs: Vec::new(),
            duplicate_compute_budget: false,
        };
        let mut high_fee = context(&source, &config, &high_fee_decoded, &[], &[]);
        high_fee.execution_units = Some(1);
        let high_fee_ids: Vec<_> = evaluate(&high_fee)
            .findings
            .into_iter()
            .map(|finding| finding.rule_id)
            .collect();
        assert!(high_fee_ids.iter().any(|rule| rule == "FEE-002"));
        assert!(high_fee_ids.iter().any(|rule| rule == "FEE-004"));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn authority_account_and_token_2022_rule_families_have_positive_cases() {
        let config = config();
        let source = TransactionSource::Serialized {
            transaction_base64: "AQ==".to_owned(),
        };
        let wallet = Address32::new([1; 32]);
        let destination = Address32::new([2; 32]);
        let token: Address32 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        let loader: Address32 = "BPFLoaderUpgradeab1e11111111111111111111111"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        let action =
            |index: usize, kind: &str, details: BTreeMap<String, serde_json::Value>| ActionData {
                instruction_index: index,
                kind: kind.to_owned(),
                program_id: token,
                accounts: vec![wallet, destination],
                details,
                known: true,
            };
        let decoded = DecodeSummary {
            actions: vec![
                action(
                    0,
                    "set_authority",
                    BTreeMap::from([("authority_type".to_owned(), serde_json::json!(0))]),
                ),
                action(
                    1,
                    "set_authority",
                    BTreeMap::from([("authority_type".to_owned(), serde_json::json!(1))]),
                ),
                action(
                    2,
                    "set_authority",
                    BTreeMap::from([("authority_type".to_owned(), serde_json::json!(2))]),
                ),
                action(
                    3,
                    "approve",
                    BTreeMap::from([("amount_raw".to_owned(), serde_json::json!("1"))]),
                ),
                action(4, "authorize_nonce_account", BTreeMap::new()),
                action(5, "close_account", BTreeMap::new()),
                action(6, "freeze_account", BTreeMap::new()),
                action(7, "burn", BTreeMap::new()),
                action(8, "mint_to", BTreeMap::new()),
                ActionData {
                    instruction_index: 9,
                    kind: "set_authority_checked".to_owned(),
                    program_id: loader,
                    accounts: vec![wallet, destination],
                    details: BTreeMap::new(),
                    known: true,
                },
                ActionData {
                    instruction_index: 10,
                    kind: "close".to_owned(),
                    program_id: loader,
                    accounts: vec![wallet],
                    details: BTreeMap::new(),
                    known: true,
                },
            ],
            decoded: 11,
            unknown_programs: Vec::new(),
            duplicate_compute_budget: false,
        };
        let extensions = [
            "permanent_delegate".to_owned(),
            "transfer_hook".to_owned(),
            "transfer_fee_config".to_owned(),
            "confidential_transfer_mint".to_owned(),
            "default_account_state".to_owned(),
            "non_transferable".to_owned(),
            "cpi_guard".to_owned(),
        ];
        let wallets = [wallet];
        let mut risk_context = context(&source, &config, &decoded, &[], &wallets);
        risk_context.token_2022_extensions = &extensions;
        let ids: Vec<_> = evaluate(&risk_context)
            .findings
            .into_iter()
            .map(|finding| finding.rule_id)
            .collect();
        for required in [
            "AUTH-001", "AUTH-002", "AUTH-003", "AUTH-004", "AUTH-005", "AUTH-006", "AUTH-007",
            "AUTH-008", "ACCT-001", "ACCT-002", "ACCT-003", "ACCT-004", "ACCT-005", "T22-001",
            "T22-002", "T22-003", "T22-004", "T22-005", "T22-006", "PROG-004", "PROG-005",
        ] {
            assert!(
                ids.iter().any(|rule| rule == required),
                "missing {required}"
            );
        }
    }

    #[test]
    fn transfer_and_intent_rule_families_have_positive_cases() {
        let mut config = config();
        let source = TransactionSource::Serialized {
            transaction_base64: "AQ==".to_owned(),
        };
        let wallet = Address32::new([1; 32]);
        let blocked = Address32::new([2; 32]);
        let unknown_recipient = Address32::new([3; 32]);
        let declared_recipient = Address32::new([4; 32]);
        let system: Address32 = "11111111111111111111111111111111"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        let token: Address32 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        config.policy.blocked_recipients.insert(blocked);
        let transfer = |index: usize, recipient: Address32| ActionData {
            instruction_index: index,
            kind: "transfer".to_owned(),
            program_id: system,
            accounts: vec![wallet, recipient],
            details: BTreeMap::from([("lamports".to_owned(), serde_json::json!("200000000"))]),
            known: true,
        };
        let decoded = DecodeSummary {
            actions: vec![
                transfer(0, blocked),
                transfer(1, unknown_recipient),
                ActionData {
                    instruction_index: 2,
                    kind: "approve".to_owned(),
                    program_id: token,
                    accounts: vec![wallet, unknown_recipient],
                    details: BTreeMap::from([("amount_raw".to_owned(), serde_json::json!("1"))]),
                    known: true,
                },
            ],
            decoded: 3,
            unknown_programs: Vec::new(),
            duplicate_compute_budget: false,
        };
        let intent = ExpectedIntent {
            description: None,
            allowed_programs: vec![Address32::new([5; 32])],
            allowed_recipients: vec![declared_recipient],
            max_sol_out_lamports: Some("100000000".to_owned()),
            token_limits: Vec::new(),
        };
        let mint = Address32::new([6; 32]);
        let deltas = [AssetDeltaData {
            account: wallet,
            owner: Some(wallet),
            mint: Some(mint),
            program_id: Some(token),
            asset: "token".to_owned(),
            raw_delta: -(i128::from(u64::MAX) + 1),
            decimals: Some(0),
        }];
        let wallets = [wallet];
        let mut risk_context = context(&source, &config, &decoded, &deltas, &wallets);
        risk_context.expected_intent = Some(&intent);
        let ids: Vec<_> = evaluate(&risk_context)
            .findings
            .into_iter()
            .map(|finding| finding.rule_id)
            .collect();

        for required in [
            "XFER-002", "XFER-003", "XFER-004", "XFER-005", "INT-001", "INT-002", "INT-003",
            "INT-005",
        ] {
            assert!(
                ids.iter().any(|rule| rule == required),
                "missing {required}"
            );
        }
        assert!(
            !ids.iter().any(|rule| rule == "XFER-001"),
            "token deltas must not be counted as SOL outflow"
        );
    }
}
