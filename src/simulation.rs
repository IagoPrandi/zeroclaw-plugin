use std::collections::HashMap;

use base64::Engine as _;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    address::Address32,
    decoders::{ActionData, decode_instruction},
    error::GuardianError,
    output::{AssetDelta, ExecutionStatus, ReturnDataReport},
    transaction::{NormalizedTransaction, ResolvedInstruction},
};

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionEffects {
    pub status: ExecutionStatus,
    pub error: Option<String>,
    pub units_consumed: Option<u64>,
    pub logs: Vec<String>,
    pub logs_truncated: bool,
    pub inner_actions: Vec<InnerAction>,
    pub asset_deltas: Vec<AssetDeltaData>,
    pub fee_lamports: Option<u64>,
    pub return_data: Option<ReturnDataReport>,
    pub post_sol_balances: Vec<(Address32, u64)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InnerAction {
    pub top_level_index: usize,
    pub inner_index: usize,
    pub stack_height: Option<u32>,
    pub action: ActionData,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetDeltaData {
    pub account: Address32,
    pub owner: Option<Address32>,
    pub mint: Option<Address32>,
    pub program_id: Option<Address32>,
    pub asset: String,
    pub raw_delta: i128,
    pub decimals: Option<u8>,
}

impl AssetDeltaData {
    #[must_use]
    pub fn into_output(self) -> AssetDelta {
        AssetDelta {
            account: self.account,
            owner: self.owner,
            mint: self.mint,
            program_id: self.program_id,
            asset: self.asset,
            raw_delta: self.raw_delta.to_string(),
            decimals: self.decimals,
        }
    }
}

/// Parse confirmed transaction metadata and reconcile effective deltas.
///
/// # Errors
///
/// Returns a typed protocol/overflow/index error for inconsistent metadata.
pub fn confirmed_effects(
    meta: &Value,
    transaction: &NormalizedTransaction,
) -> Result<ExecutionEffects, GuardianError> {
    let error_value = meta.get("err").cloned().unwrap_or(Value::Null);
    let failed = !error_value.is_null();
    let pre_balances = u64_array(meta, "preBalances")?;
    let post_balances = u64_array(meta, "postBalances")?;
    let mut deltas = sol_deltas(&pre_balances, &post_balances, transaction)?;
    deltas.extend(token_deltas(
        meta.get("preTokenBalances"),
        meta.get("postTokenBalances"),
        transaction,
    )?);
    Ok(ExecutionEffects {
        status: if failed {
            ExecutionStatus::ConfirmedFailed
        } else {
            ExecutionStatus::ConfirmedSucceeded
        },
        error: failed.then(|| compact_error(&error_value)),
        units_consumed: meta.get("computeUnitsConsumed").and_then(Value::as_u64),
        logs: string_array(meta.get("logMessages")),
        logs_truncated: false,
        inner_actions: decode_inner_instructions(meta.get("innerInstructions"), transaction)?,
        asset_deltas: deltas,
        fee_lamports: meta.get("fee").and_then(Value::as_u64),
        return_data: parse_return_data(meta.get("returnData"))?,
        post_sol_balances: post_balances
            .iter()
            .copied()
            .zip(&transaction.account_keys)
            .map(|(balance, account)| (account.address, balance))
            .collect(),
    })
}

/// Parse a `simulateTransaction` result.
///
/// # Errors
///
/// Returns a typed protocol/index error for malformed execution metadata.
pub fn simulation_effects(
    result: &Value,
    transaction: &NormalizedTransaction,
) -> Result<ExecutionEffects, GuardianError> {
    let value = result.get("value").unwrap_or(result);
    let error_value = value.get("err").cloned().unwrap_or(Value::Null);
    let failed = !error_value.is_null();
    Ok(ExecutionEffects {
        status: if failed {
            ExecutionStatus::SimulationFailed
        } else {
            ExecutionStatus::SimulationSucceeded
        },
        error: failed.then(|| compact_error(&error_value)),
        units_consumed: value.get("unitsConsumed").and_then(Value::as_u64),
        logs: string_array(value.get("logs")),
        logs_truncated: false,
        inner_actions: decode_inner_instructions(value.get("innerInstructions"), transaction)?,
        asset_deltas: Vec::new(),
        fee_lamports: None,
        return_data: parse_return_data(value.get("returnData"))?,
        post_sol_balances: Vec::new(),
    })
}

fn parse_return_data(value: Option<&Value>) -> Result<Option<ReturnDataReport>, GuardianError> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    let program_id = value
        .get("programId")
        .and_then(Value::as_str)
        .ok_or(GuardianError::RpcProtocol)?
        .parse()
        .map_err(|_| GuardianError::RpcProtocol)?;
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or(GuardianError::RpcProtocol)?;
    let encoded = data
        .first()
        .and_then(Value::as_str)
        .ok_or(GuardianError::RpcProtocol)?;
    if data.get(1).and_then(Value::as_str) != Some("base64") {
        return Err(GuardianError::RpcProtocol);
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| GuardianError::RpcProtocol)?;
    Ok(Some(ReturnDataReport {
        program_id,
        encoding: "base64".to_owned(),
        data_length: decoded.len(),
        data_sha256: hex::encode(Sha256::digest(&decoded)),
    }))
}

/// Compute the priority fee from requested limit and micro-lamport price.
///
/// # Errors
///
/// Returns `ArithmeticOverflow` if multiplication or conversion is unsafe.
pub fn priority_fee_lamports(limit: u64, micro_lamports: u64) -> Result<u64, GuardianError> {
    let product = u128::from(limit)
        .checked_mul(u128::from(micro_lamports))
        .ok_or(GuardianError::ArithmeticOverflow)?;
    let rounded = product
        .checked_add(999_999)
        .ok_or(GuardianError::ArithmeticOverflow)?
        / 1_000_000;
    u64::try_from(rounded).map_err(|_| GuardianError::ArithmeticOverflow)
}

#[must_use]
pub fn has_durable_nonce(actions: &[ActionData]) -> bool {
    actions
        .iter()
        .any(|action| action.kind == "advance_nonce_account")
}

fn sol_deltas(
    pre: &[u64],
    post: &[u64],
    transaction: &NormalizedTransaction,
) -> Result<Vec<AssetDeltaData>, GuardianError> {
    if pre.len() != post.len() || pre.len() != transaction.account_keys.len() {
        return Err(GuardianError::RpcProtocol);
    }
    Ok(pre
        .iter()
        .zip(post)
        .zip(&transaction.account_keys)
        .filter_map(|((before, after), account)| {
            let delta = i128::from(*after) - i128::from(*before);
            (delta != 0).then_some(AssetDeltaData {
                account: account.address,
                owner: account.owner,
                mint: None,
                program_id: None,
                asset: "SOL".to_owned(),
                raw_delta: delta,
                decimals: Some(9),
            })
        })
        .collect())
}

type TokenKey = (usize, Address32, Option<Address32>, Option<Address32>);

#[derive(Clone, Copy)]
struct TokenAmount {
    raw: u64,
    decimals: Option<u8>,
}

fn token_deltas(
    pre: Option<&Value>,
    post: Option<&Value>,
    transaction: &NormalizedTransaction,
) -> Result<Vec<AssetDeltaData>, GuardianError> {
    let pre = token_map(pre)?;
    let post = token_map(post)?;
    let mut keys: Vec<_> = pre.keys().chain(post.keys()).copied().collect();
    keys.sort_unstable();
    keys.dedup();
    keys.into_iter()
        .filter_map(|key| {
            let before = pre.get(&key).copied().unwrap_or(TokenAmount {
                raw: 0,
                decimals: post.get(&key).and_then(|amount| amount.decimals),
            });
            let after = post.get(&key).copied().unwrap_or(TokenAmount {
                raw: 0,
                decimals: before.decimals,
            });
            let delta = i128::from(after.raw) - i128::from(before.raw);
            if delta == 0 {
                return None;
            }
            let account = transaction
                .account_keys
                .get(key.0)
                .map(|account| account.address)
                .ok_or(GuardianError::InvalidAccountIndex);
            Some(account.map(|account| AssetDeltaData {
                account,
                owner: key.2,
                mint: Some(key.1),
                program_id: key.3,
                asset: "token".to_owned(),
                raw_delta: delta,
                decimals: after.decimals.or(before.decimals),
            }))
        })
        .collect()
}

fn token_map(value: Option<&Value>) -> Result<HashMap<TokenKey, TokenAmount>, GuardianError> {
    let mut map = HashMap::new();
    for balance in value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let index = balance
            .get("accountIndex")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(GuardianError::RpcProtocol)?;
        let mint = parse_address(balance.get("mint"))?.ok_or(GuardianError::RpcProtocol)?;
        let owner = parse_address(balance.get("owner"))?;
        let program = parse_address(balance.get("programId"))?;
        let amount = balance
            .pointer("/uiTokenAmount/amount")
            .and_then(Value::as_str)
            .ok_or(GuardianError::RpcProtocol)?
            .parse()
            .map_err(|_| GuardianError::RpcProtocol)?;
        let decimals = balance
            .pointer("/uiTokenAmount/decimals")
            .and_then(Value::as_u64)
            .and_then(|value| u8::try_from(value).ok());
        if map
            .insert(
                (index, mint, owner, program),
                TokenAmount {
                    raw: amount,
                    decimals,
                },
            )
            .is_some()
        {
            return Err(GuardianError::RpcProtocol);
        }
    }
    Ok(map)
}

fn decode_inner_instructions(
    value: Option<&Value>,
    transaction: &NormalizedTransaction,
) -> Result<Vec<InnerAction>, GuardianError> {
    let mut output = Vec::new();
    for group in value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let top_level_index = group
            .get("index")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(GuardianError::RpcProtocol)?;
        let instructions = group
            .get("instructions")
            .and_then(Value::as_array)
            .ok_or(GuardianError::RpcProtocol)?;
        for (inner_index, raw) in instructions.iter().enumerate() {
            let program_index = raw
                .get("programIdIndex")
                .and_then(Value::as_u64)
                .and_then(|value| usize::try_from(value).ok())
                .ok_or(GuardianError::RpcProtocol)?;
            let account_indexes = raw
                .get("accounts")
                .and_then(Value::as_array)
                .ok_or(GuardianError::RpcProtocol)?
                .iter()
                .map(|value| {
                    value
                        .as_u64()
                        .and_then(|value| usize::try_from(value).ok())
                        .filter(|index| *index < transaction.account_keys.len())
                        .ok_or(GuardianError::InvalidAccountIndex)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let data = raw
                .get("data")
                .and_then(Value::as_str)
                .ok_or(GuardianError::RpcProtocol)
                .and_then(|value| {
                    bs58::decode(value)
                        .into_vec()
                        .map_err(|_| GuardianError::RpcProtocol)
                })?;
            let program_id = transaction
                .account_keys
                .get(program_index)
                .map(|account| account.address)
                .ok_or(GuardianError::InvalidAccountIndex)?;
            let accounts = account_indexes
                .iter()
                .map(|index| transaction.account_keys[*index].address)
                .collect();
            let instruction = ResolvedInstruction {
                instruction_index: top_level_index,
                program_id,
                program_id_index: program_index,
                accounts: account_indexes,
                data,
            };
            output.push(InnerAction {
                top_level_index,
                inner_index,
                stack_height: raw
                    .get("stackHeight")
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok()),
                action: decode_instruction(&instruction, accounts)?,
            });
        }
    }
    Ok(output)
}

fn u64_array(object: &Value, key: &str) -> Result<Vec<u64>, GuardianError> {
    object
        .get(key)
        .and_then(Value::as_array)
        .ok_or(GuardianError::RpcProtocol)?
        .iter()
        .map(|value| value.as_u64().ok_or(GuardianError::RpcProtocol))
        .collect()
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .take(1_000)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_address(value: Option<&Value>) -> Result<Option<Address32>, GuardianError> {
    value
        .and_then(Value::as_str)
        .map(str::parse)
        .transpose()
        .map_err(|_| GuardianError::RpcProtocol)
}

fn compact_error(value: &Value) -> String {
    let serialized = value.to_string();
    serialized.chars().take(500).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use solana_message::{Message, MessageHeader, VersionedMessage};
    use solana_transaction::versioned::VersionedTransaction;

    use super::{confirmed_effects, priority_fee_lamports, simulation_effects};
    use crate::{limits::Limits, transaction::normalize_wire};

    fn transaction() -> crate::transaction::NormalizedTransaction {
        let wire = bincode::serialize(&VersionedTransaction {
            signatures: vec![solana_transaction::Signature::default()],
            message: VersionedMessage::Legacy(Message {
                header: MessageHeader {
                    num_required_signatures: 1,
                    num_readonly_signed_accounts: 0,
                    num_readonly_unsigned_accounts: 1,
                },
                account_keys: vec![
                    solana_message::Address::new_from_array([1; 32]),
                    "11111111111111111111111111111111"
                        .parse()
                        .unwrap_or_else(|_| solana_message::Address::new_from_array([2; 32])),
                ],
                recent_blockhash: solana_message::Hash::default(),
                instructions: vec![],
            }),
        })
        .unwrap_or_default();
        normalize_wire(
            &wire,
            &Limits {
                max_rpc_calls: 8,
                max_http_response_bytes: 4096,
                max_transaction_bytes: 1232,
                max_output_bytes: 8192,
                max_accounts: 512,
                max_instructions: 256,
            },
            &HashMap::new(),
        )
        .unwrap_or_else(|_| unreachable!())
    }

    #[test]
    fn reconciles_confirmed_sol_and_token_deltas() {
        let tx = transaction();
        let mint = solana_message::Address::new_from_array([9; 32]).to_string();
        let owner = solana_message::Address::new_from_array([8; 32]).to_string();
        let meta = serde_json::json!({
            "err": null,
            "fee": 5000,
            "preBalances": [10000, 20],
            "postBalances": [5000, 5020],
            "preTokenBalances": [{
                "accountIndex": 1, "mint": mint, "owner": owner,
                "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "uiTokenAmount": {"amount":"10","decimals":2}
            }],
            "postTokenBalances": [{
                "accountIndex": 1, "mint": mint, "owner": owner,
                "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "uiTokenAmount": {"amount":"25","decimals":2}
            }],
            "logMessages": [],
            "innerInstructions": []
        });
        let effects = confirmed_effects(&meta, &tx).unwrap_or_else(|_| unreachable!());
        assert_eq!(effects.asset_deltas.len(), 3);
        assert!(
            effects
                .asset_deltas
                .iter()
                .any(|delta| delta.raw_delta == 15)
        );
        assert_eq!(effects.fee_lamports, Some(5000));
    }

    #[test]
    fn simulation_failure_is_not_success() {
        let result = serde_json::json!({
            "value": {"err":{"InstructionError":[0,"Custom"]},"logs":[],"innerInstructions":[]}
        });
        let effects = simulation_effects(&result, &transaction());
        assert!(matches!(
            effects.map(|value| value.status),
            Ok(crate::output::ExecutionStatus::SimulationFailed)
        ));
    }

    #[test]
    fn captures_bounded_return_data_evidence() {
        let tx = transaction();
        let result = serde_json::json!({
            "value": {
                "err": null,
                "returnData": {
                    "programId": "11111111111111111111111111111111",
                    "data": ["AQID", "base64"]
                }
            }
        });

        let effects = simulation_effects(&result, &tx).unwrap_or_else(|_| unreachable!());
        let return_data = effects.return_data.unwrap_or_else(|| unreachable!());

        assert_eq!(return_data.data_length, 3);
        assert_eq!(
            return_data.data_sha256,
            "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81"
        );
    }

    #[test]
    fn rejects_malformed_return_data() {
        let tx = transaction();
        let result = serde_json::json!({
            "value": {
                "err": null,
                "returnData": {
                    "programId": "11111111111111111111111111111111",
                    "data": ["not-base64", "base64"]
                }
            }
        });

        assert!(simulation_effects(&result, &tx).is_err());
    }

    #[test]
    fn decodes_inner_instruction_with_stack_height() {
        let mut system_transfer = 2_u32.to_le_bytes().to_vec();
        system_transfer.extend_from_slice(&1_u64.to_le_bytes());
        let result = serde_json::json!({
            "value": {
                "err": null,
                "logs": [],
                "innerInstructions": [{
                    "index": 0,
                    "instructions": [{
                        "programIdIndex": 1,
                        "accounts": [0],
                        "data": bs58::encode(system_transfer).into_string(),
                        "stackHeight": 2
                    }]
                }]
            }
        });
        let effects =
            simulation_effects(&result, &transaction()).unwrap_or_else(|_| unreachable!());
        assert_eq!(effects.inner_actions[0].action.kind, "transfer");
        assert_eq!(effects.inner_actions[0].stack_height, Some(2));
    }

    #[test]
    fn priority_fee_rounds_up() {
        assert_eq!(priority_fee_lamports(3, 500_001), Ok(2));
    }
}
