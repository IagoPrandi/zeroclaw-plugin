use std::collections::{BTreeMap, BTreeSet};

use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::{
    address::Address32,
    decoders::{ActionData, is_token_2022_program, is_token_program, token_2022_extension_names},
    error::GuardianError,
    rpc::{RpcClient, RpcTransport},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CriticalStateIssue {
    pub account: Address32,
    pub reason: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerMismatch {
    pub account: Address32,
    pub expected_owner: Address32,
    pub actual_owner: Address32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StateEvidence {
    pub token_2022_extensions: Vec<String>,
    pub critical_issues: Vec<CriticalStateIssue>,
    pub owner_mismatches: Vec<OwnerMismatch>,
    pub sol_balances: Vec<(Address32, u64)>,
    pub rpc_inconsistent: bool,
}

/// Fetch the minimum account state needed by token and observed-wallet rules.
///
/// # Errors
///
/// Returns a typed RPC error when transport or the validated JSON-RPC envelope
/// fails. Inconsistent result contents remain explicit fail-closed evidence.
#[allow(clippy::too_many_lines)]
pub fn load_required_state<T: RpcTransport>(
    rpc: &mut RpcClient<T>,
    actions: &[ActionData],
    observed_wallets: &[Address32],
    min_context_slot: Option<u64>,
    confirmed: bool,
) -> Result<StateEvidence, GuardianError> {
    let created = created_accounts(actions);
    let mut expected_owners: BTreeMap<Address32, BTreeSet<Address32>> = BTreeMap::new();
    for action in actions {
        if !is_token_program(action.program_id) && !is_token_2022_program(action.program_id) {
            continue;
        }
        for index in token_state_indexes(action.kind.as_str()) {
            if confirmed && action.kind == "close_account" && index == 0 {
                continue;
            }
            if let Some(account) = action.accounts.get(index) {
                expected_owners
                    .entry(*account)
                    .or_default()
                    .insert(action.program_id);
            }
        }
    }
    let mut requested: BTreeSet<Address32> = observed_wallets.iter().copied().collect();
    requested.extend(expected_owners.keys().copied());
    if requested.is_empty() {
        return Ok(StateEvidence::default());
    }
    let addresses: Vec<_> = requested.iter().map(ToString::to_string).collect();
    let result = rpc.get_multiple_accounts(&addresses, "confirmed", min_context_slot)?;
    let Some(accounts) = result.get("value").and_then(serde_json::Value::as_array) else {
        return Ok(StateEvidence {
            rpc_inconsistent: true,
            ..StateEvidence::default()
        });
    };
    if accounts.len() != addresses.len() {
        return Ok(StateEvidence {
            rpc_inconsistent: true,
            ..StateEvidence::default()
        });
    }

    let mut evidence = StateEvidence::default();
    for (address, account) in requested.iter().copied().zip(accounts) {
        if account.is_null() {
            if expected_owners.contains_key(&address) && !created.contains(&address) {
                evidence.critical_issues.push(CriticalStateIssue {
                    account: address,
                    reason: "account_not_found",
                });
            }
            continue;
        }
        if observed_wallets.contains(&address) {
            match account.get("lamports").and_then(serde_json::Value::as_u64) {
                Some(lamports) => evidence.sol_balances.push((address, lamports)),
                None => evidence.rpc_inconsistent = true,
            }
        }
        let Some(expected) = expected_owners.get(&address) else {
            continue;
        };
        let Some(actual_owner) = account
            .get("owner")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse().ok())
        else {
            evidence.rpc_inconsistent = true;
            continue;
        };
        for expected_owner in expected {
            if *expected_owner != actual_owner {
                evidence.owner_mismatches.push(OwnerMismatch {
                    account: address,
                    expected_owner: *expected_owner,
                    actual_owner,
                });
            }
        }
        if !expected.iter().any(|owner| is_token_2022_program(*owner))
            || !is_token_2022_program(actual_owner)
        {
            continue;
        }
        let Some(encoded) = account
            .pointer("/data/0")
            .and_then(serde_json::Value::as_str)
        else {
            evidence.rpc_inconsistent = true;
            continue;
        };
        let Ok(data) = STANDARD.decode(encoded) else {
            evidence.critical_issues.push(CriticalStateIssue {
                account: address,
                reason: "invalid_base64_account_data",
            });
            continue;
        };
        if !matches!(data.len(), 82 | 165 | 166) && data.len() < 167 {
            evidence.critical_issues.push(CriticalStateIssue {
                account: address,
                reason: "invalid_token_state_length",
            });
            continue;
        }
        if data.len() >= 166 {
            match token_2022_extension_names(&data, 166) {
                Ok(extensions) => evidence.token_2022_extensions.extend(extensions),
                Err(_) => evidence.critical_issues.push(CriticalStateIssue {
                    account: address,
                    reason: "malformed_token_2022_tlv",
                }),
            }
        }
    }
    evidence.token_2022_extensions.sort();
    evidence.token_2022_extensions.dedup();
    evidence.critical_issues.sort_by_key(|issue| issue.account);
    evidence
        .owner_mismatches
        .sort_by_key(|issue| (issue.account, issue.expected_owner, issue.actual_owner));
    evidence.sol_balances.sort_unstable();
    Ok(evidence)
}

fn created_accounts(actions: &[ActionData]) -> BTreeSet<Address32> {
    actions
        .iter()
        .filter(|action| {
            matches!(
                action.kind.as_str(),
                "create_account"
                    | "create_associated_token_account"
                    | "create_associated_token_account_idempotent"
            )
        })
        .filter_map(|action| action.accounts.get(1).copied())
        .collect()
}

fn token_state_indexes(kind: &str) -> Vec<usize> {
    match kind {
        "transfer_checked" => vec![0, 1, 2],
        "approve" | "revoke" | "set_authority" | "close_account" | "sync_native" => vec![0],
        "approve_checked" | "mint_to" | "burn" | "freeze_account" | "thaw_account" => {
            vec![0, 1]
        }
        "transfer" | "mint_to_checked" | "burn_checked" => vec![0, 1],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, VecDeque};

    use base64::{Engine as _, engine::general_purpose::STANDARD};

    use super::load_required_state;
    use crate::{
        address::Address32,
        decoders::ActionData,
        limits::{Budget, Limits},
        rpc::{HttpResponse, RpcClient, RpcTransport, TransportError},
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

    fn client(body: String) -> RpcClient<MockTransport> {
        RpcClient::new(
            "https://rpc.invalid".to_owned(),
            5_000,
            MockTransport {
                responses: VecDeque::from([HttpResponse {
                    status: 200,
                    body: body.into_bytes(),
                }]),
            },
            Budget::new(Limits {
                max_rpc_calls: 8,
                max_http_response_bytes: 16_384,
                max_transaction_bytes: 1_232,
                max_output_bytes: 16_384,
                max_accounts: 512,
                max_instructions: 256,
            }),
        )
    }

    #[test]
    fn extracts_balances_and_owner_mismatches() {
        let token_2022: Address32 = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        let account = Address32::new([7; 32]);
        let observed = Address32::new([8; 32]);
        let wrong_owner = Address32::new([9; 32]);
        let mut data = vec![0_u8; 166];
        data.extend_from_slice(&14_u16.to_le_bytes());
        data.extend_from_slice(&0_u16.to_le_bytes());
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "value": [
                    {
                        "lamports": 1,
                        "owner": wrong_owner.to_string(),
                        "data": [STANDARD.encode(data), "base64"]
                    },
                    {
                        "lamports": 42,
                        "owner": wrong_owner.to_string(),
                        "data": ["", "base64"]
                    }
                ]
            }
        })
        .to_string();
        let action = ActionData {
            instruction_index: 0,
            kind: "transfer".to_owned(),
            program_id: token_2022,
            accounts: vec![account, account],
            details: BTreeMap::default(),
            known: true,
        };
        let mut rpc = client(body);

        let evidence = load_required_state(&mut rpc, &[action], &[observed], None, false)
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(evidence.sol_balances, vec![(observed, 42)]);
        assert_eq!(evidence.owner_mismatches.len(), 1);
    }

    #[test]
    fn extracts_token_2022_extensions_from_account_state() {
        let token_2022: Address32 = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
            .parse()
            .unwrap_or_else(|_| unreachable!());
        let account = Address32::new([7; 32]);
        let mut data = vec![0_u8; 166];
        data.extend_from_slice(&14_u16.to_le_bytes());
        data.extend_from_slice(&0_u16.to_le_bytes());
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "value": [{
                    "lamports": 1,
                    "owner": token_2022.to_string(),
                    "data": [STANDARD.encode(data), "base64"]
                }]
            }
        })
        .to_string();
        let action = ActionData {
            instruction_index: 0,
            kind: "revoke".to_owned(),
            program_id: token_2022,
            accounts: vec![account],
            details: BTreeMap::new(),
            known: true,
        };
        let mut rpc = client(body);

        let evidence = load_required_state(&mut rpc, &[action], &[], None, false)
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(
            evidence.token_2022_extensions,
            vec!["transfer_hook".to_owned()]
        );
        assert!(evidence.critical_issues.is_empty());
    }
}
