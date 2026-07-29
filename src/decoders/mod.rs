use std::collections::BTreeMap;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    address::Address32,
    error::GuardianError,
    output::Action,
    transaction::{NormalizedTransaction, ResolvedInstruction},
};

const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
const COMPUTE_BUDGET_PROGRAM: &str = "ComputeBudget111111111111111111111111111111";
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const ASSOCIATED_TOKEN_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const MEMO_PROGRAM: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";
const ALT_PROGRAM: &str = "AddressLookupTab1e1111111111111111111111111";
const UPGRADEABLE_LOADER: &str = "BPFLoaderUpgradeab1e11111111111111111111111";

#[must_use]
pub fn is_token_program(program_id: Address32) -> bool {
    program_id.to_string() == TOKEN_PROGRAM
}

#[must_use]
pub fn is_token_2022_program(program_id: Address32) -> bool {
    program_id.to_string() == TOKEN_2022_PROGRAM
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DecodeSummary {
    pub actions: Vec<ActionData>,
    pub decoded: usize,
    pub unknown_programs: Vec<Address32>,
    pub duplicate_compute_budget: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionData {
    pub instruction_index: usize,
    pub kind: String,
    pub program_id: Address32,
    pub accounts: Vec<Address32>,
    pub details: BTreeMap<String, Value>,
    pub known: bool,
}

impl ActionData {
    #[must_use]
    pub fn into_output(self) -> Action {
        Action {
            instruction_index: self.instruction_index,
            inner_index: None,
            kind: self.kind,
            program_id: self.program_id,
            accounts: self.accounts,
            details: self.details,
        }
    }
}

/// Decode every top-level instruction through the static program registry.
///
/// # Errors
///
/// Returns a controlled decoder error when a known instruction is malformed or
/// its resolved account indices are inconsistent.
pub fn decode_transaction(
    transaction: &NormalizedTransaction,
) -> Result<DecodeSummary, GuardianError> {
    let mut summary = DecodeSummary::default();
    let mut compute_kinds = Vec::new();
    for instruction in &transaction.instructions {
        let accounts = instruction
            .accounts
            .iter()
            .map(|index| {
                transaction
                    .account_keys
                    .get(*index)
                    .map(|key| key.address)
                    .ok_or(GuardianError::InvalidAccountIndex)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let action = decode_instruction(instruction, accounts)?;
        if action.known {
            summary.decoded += 1;
        } else if !summary.unknown_programs.contains(&action.program_id) {
            summary.unknown_programs.push(action.program_id);
        }
        if instruction.program_id.to_string() == COMPUTE_BUDGET_PROGRAM {
            if compute_kinds.contains(&action.kind) {
                summary.duplicate_compute_budget = true;
            }
            compute_kinds.push(action.kind.clone());
        }
        summary.actions.push(action);
    }
    summary.unknown_programs.sort_unstable();
    Ok(summary)
}

pub(crate) fn decode_instruction(
    instruction: &ResolvedInstruction,
    accounts: Vec<Address32>,
) -> Result<ActionData, GuardianError> {
    let program = instruction.program_id.to_string();
    let (kind, details, known) = match program.as_str() {
        SYSTEM_PROGRAM => decode_system(&instruction.data)?,
        COMPUTE_BUDGET_PROGRAM => decode_compute_budget(&instruction.data)?,
        TOKEN_PROGRAM | TOKEN_2022_PROGRAM => decode_token(&instruction.data)?,
        ASSOCIATED_TOKEN_PROGRAM => decode_associated_token(&instruction.data)?,
        MEMO_PROGRAM => decode_memo(&instruction.data),
        ALT_PROGRAM => decode_discriminant(
            &instruction.data,
            &[
                "create_lookup_table",
                "freeze_lookup_table",
                "extend_lookup_table",
                "deactivate_lookup_table",
                "close_lookup_table",
            ],
        )?,
        UPGRADEABLE_LOADER => decode_discriminant(
            &instruction.data,
            &[
                "initialize_buffer",
                "write",
                "deploy_with_max_data_len",
                "upgrade",
                "set_authority",
                "close",
                "extend_program",
                "set_authority_checked",
            ],
        )?,
        _ => {
            let mut details = BTreeMap::new();
            details.insert("data_length".to_owned(), json!(instruction.data.len()));
            details.insert(
                "data_sha256".to_owned(),
                json!(hex::encode(Sha256::digest(&instruction.data))),
            );
            ("unknown_program".to_owned(), details, false)
        }
    };
    Ok(ActionData {
        instruction_index: instruction.instruction_index,
        kind,
        program_id: instruction.program_id,
        accounts,
        details,
        known,
    })
}

fn decode_system(data: &[u8]) -> Result<(String, BTreeMap<String, Value>, bool), GuardianError> {
    let variants = [
        "create_account",
        "assign",
        "transfer",
        "create_account_with_seed",
        "advance_nonce_account",
        "withdraw_nonce_account",
        "initialize_nonce_account",
        "authorize_nonce_account",
        "allocate",
        "allocate_with_seed",
        "assign_with_seed",
        "transfer_with_seed",
        "upgrade_nonce_account",
    ];
    let (kind, mut details, known) = decode_discriminant(data, &variants)?;
    if matches!(
        kind.as_str(),
        "create_account"
            | "transfer"
            | "withdraw_nonce_account"
            | "allocate"
            | "transfer_with_seed"
    ) {
        let offset = if kind == "create_account" { 12 } else { 4 };
        if let Some(amount) = read_u64(data, offset) {
            details.insert(
                if kind == "allocate" {
                    "space"
                } else {
                    "lamports"
                }
                .to_owned(),
                json!(amount.to_string()),
            );
        }
    }
    if kind == "authorize_nonce_account" {
        let bytes = data
            .get(4..36)
            .and_then(|value| value.try_into().ok())
            .ok_or(GuardianError::Decoder)?;
        details.insert(
            "new_authority".to_owned(),
            json!(Address32::new(bytes).to_string()),
        );
    }
    Ok((kind, details, known))
}

fn decode_compute_budget(
    data: &[u8],
) -> Result<(String, BTreeMap<String, Value>, bool), GuardianError> {
    let Some(tag) = data.first().copied() else {
        return Err(GuardianError::Decoder);
    };
    let mut details = BTreeMap::new();
    let kind = match tag {
        1 => {
            details.insert(
                "bytes".to_owned(),
                json!(read_u32(data, 1).ok_or(GuardianError::Decoder)?.to_string()),
            );
            "request_heap_frame"
        }
        2 => {
            details.insert(
                "units".to_owned(),
                json!(read_u32(data, 1).ok_or(GuardianError::Decoder)?.to_string()),
            );
            "set_compute_unit_limit"
        }
        3 => {
            details.insert(
                "micro_lamports".to_owned(),
                json!(read_u64(data, 1).ok_or(GuardianError::Decoder)?.to_string()),
            );
            "set_compute_unit_price"
        }
        4 => {
            details.insert(
                "bytes".to_owned(),
                json!(read_u32(data, 1).ok_or(GuardianError::Decoder)?.to_string()),
            );
            "set_loaded_accounts_data_size_limit"
        }
        _ => return Err(GuardianError::Decoder),
    };
    Ok((kind.to_owned(), details, true))
}

fn decode_token(data: &[u8]) -> Result<(String, BTreeMap<String, Value>, bool), GuardianError> {
    let Some(tag) = data.first().copied() else {
        return Err(GuardianError::Decoder);
    };
    let names = [
        "initialize_mint",
        "initialize_account",
        "initialize_multisig",
        "transfer",
        "approve",
        "revoke",
        "set_authority",
        "mint_to",
        "burn",
        "close_account",
        "freeze_account",
        "thaw_account",
        "transfer_checked",
        "approve_checked",
        "mint_to_checked",
        "burn_checked",
        "initialize_account_2",
        "sync_native",
        "initialize_account_3",
        "initialize_multisig_2",
        "initialize_mint_2",
        "get_account_data_size",
        "initialize_immutable_owner",
    ];
    let kind = names.get(usize::from(tag)).ok_or(GuardianError::Decoder)?;
    let mut details = BTreeMap::new();
    if matches!(tag, 3 | 4 | 7 | 8 | 12 | 13 | 14 | 15) {
        details.insert(
            "amount_raw".to_owned(),
            json!(read_u64(data, 1).ok_or(GuardianError::Decoder)?.to_string()),
        );
    }
    if matches!(tag, 12..=15) {
        details.insert(
            "decimals".to_owned(),
            json!(*data.get(9).ok_or(GuardianError::Decoder)?),
        );
    }
    if tag == 6 {
        details.insert(
            "authority_type".to_owned(),
            json!(*data.get(1).ok_or(GuardianError::Decoder)?),
        );
        let option = read_u32(data, 2).ok_or(GuardianError::Decoder)?;
        match option {
            0 => {
                details.insert("new_authority".to_owned(), Value::Null);
            }
            1 => {
                let bytes = data
                    .get(6..38)
                    .and_then(|value| value.try_into().ok())
                    .ok_or(GuardianError::Decoder)?;
                details.insert(
                    "new_authority".to_owned(),
                    json!(Address32::new(bytes).to_string()),
                );
            }
            _ => return Err(GuardianError::Decoder),
        }
    }
    Ok(((*kind).to_owned(), details, true))
}

fn decode_associated_token(
    data: &[u8],
) -> Result<(String, BTreeMap<String, Value>, bool), GuardianError> {
    let kind = match data {
        [] | [0] => "create_associated_token_account",
        [1] => "create_associated_token_account_idempotent",
        [2] => "recover_nested_associated_token_account",
        _ => return Err(GuardianError::Decoder),
    };
    Ok((kind.to_owned(), BTreeMap::new(), true))
}

fn decode_memo(data: &[u8]) -> (String, BTreeMap<String, Value>, bool) {
    let mut details = BTreeMap::new();
    if data.len() <= 256
        && let Ok(memo) = std::str::from_utf8(data)
    {
        details.insert("memo".to_owned(), json!(memo));
        return ("memo".to_owned(), details, true);
    }
    details.insert("data_length".to_owned(), json!(data.len()));
    details.insert(
        "data_sha256".to_owned(),
        json!(hex::encode(Sha256::digest(data))),
    );
    ("memo_unreadable".to_owned(), details, true)
}

fn decode_discriminant(
    data: &[u8],
    variants: &[&str],
) -> Result<(String, BTreeMap<String, Value>, bool), GuardianError> {
    let tag = read_u32(data, 0).ok_or(GuardianError::Decoder)?;
    let kind = variants
        .get(usize::try_from(tag).map_err(|_| GuardianError::Decoder)?)
        .ok_or(GuardianError::Decoder)?;
    Ok(((*kind).to_owned(), BTreeMap::new(), true))
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    data.get(offset..offset.checked_add(4)?)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
}

fn read_u64(data: &[u8], offset: usize) -> Option<u64> {
    data.get(offset..offset.checked_add(8)?)
        .and_then(|value| value.try_into().ok())
        .map(u64::from_le_bytes)
}

/// List prioritized Token-2022 TLV extension names without interpreting
/// confidential payloads.
///
/// `tlv_offset` must point to the first type/length header after base state and
/// the account-type byte.
///
/// # Errors
///
/// Returns `Decoder` for truncated, overflowing, or duplicate TLV entries.
pub fn token_2022_extension_names(
    data: &[u8],
    tlv_offset: usize,
) -> Result<Vec<String>, GuardianError> {
    let names = [
        "uninitialized",
        "transfer_fee_config",
        "transfer_fee_amount",
        "mint_close_authority",
        "confidential_transfer_mint",
        "confidential_transfer_account",
        "default_account_state",
        "immutable_owner",
        "memo_transfer",
        "non_transferable",
        "interest_bearing_config",
        "cpi_guard",
        "permanent_delegate",
        "non_transferable_account",
        "transfer_hook",
        "transfer_hook_account",
        "confidential_transfer_fee_config",
        "confidential_transfer_fee_amount",
        "metadata_pointer",
        "token_metadata",
        "group_pointer",
        "token_group",
        "group_member_pointer",
        "token_group_member",
    ];
    let mut cursor = tlv_offset;
    let mut output = Vec::new();
    while cursor < data.len() {
        let extension_type = data
            .get(
                cursor
                    ..cursor
                        .checked_add(2)
                        .ok_or(GuardianError::ArithmeticOverflow)?,
            )
            .and_then(|value| value.try_into().ok())
            .map(u16::from_le_bytes)
            .ok_or(GuardianError::Decoder)?;
        let length = data
            .get(
                cursor
                    .checked_add(2)
                    .ok_or(GuardianError::ArithmeticOverflow)?
                    ..cursor
                        .checked_add(4)
                        .ok_or(GuardianError::ArithmeticOverflow)?,
            )
            .and_then(|value| value.try_into().ok())
            .map(u16::from_le_bytes)
            .map(usize::from)
            .ok_or(GuardianError::Decoder)?;
        cursor = cursor
            .checked_add(4)
            .and_then(|value| value.checked_add(length))
            .ok_or(GuardianError::ArithmeticOverflow)?;
        if cursor > data.len() {
            return Err(GuardianError::Decoder);
        }
        let name = names.get(usize::from(extension_type)).map_or_else(
            || format!("unknown_extension_{extension_type}"),
            |name| (*name).to_owned(),
        );
        if output.contains(&name) {
            return Err(GuardianError::Decoder);
        }
        output.push(name);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{
        ALT_PROGRAM, ASSOCIATED_TOKEN_PROGRAM, COMPUTE_BUDGET_PROGRAM, MEMO_PROGRAM,
        SYSTEM_PROGRAM, TOKEN_2022_PROGRAM, TOKEN_PROGRAM, UPGRADEABLE_LOADER, decode_instruction,
        token_2022_extension_names,
    };
    use crate::{address::Address32, transaction::ResolvedInstruction};

    fn instruction(program: &str, data: Vec<u8>) -> ResolvedInstruction {
        ResolvedInstruction {
            instruction_index: 0,
            program_id: program.parse().unwrap_or(Address32::ZERO),
            program_id_index: 0,
            accounts: vec![],
            data,
        }
    }

    #[test]
    fn decodes_system_transfer() {
        let mut data = 2_u32.to_le_bytes().to_vec();
        data.extend_from_slice(&42_u64.to_le_bytes());
        let action = decode_instruction(&instruction(SYSTEM_PROGRAM, data), vec![]);
        assert!(action.is_ok());
        assert_eq!(
            action
                .ok()
                .and_then(|value| value.details.get("lamports").cloned()),
            Some(serde_json::json!("42"))
        );
    }

    #[test]
    fn decodes_compute_budget_variants_and_rejects_malformed() {
        for (tag, bytes) in [(1, 4_usize), (2, 4), (3, 8), (4, 4)] {
            let mut data = vec![tag];
            data.extend(vec![0; bytes]);
            assert!(decode_instruction(&instruction(COMPUTE_BUDGET_PROGRAM, data), vec![]).is_ok());
        }
        assert!(
            decode_instruction(&instruction(COMPUTE_BUDGET_PROGRAM, vec![3, 0]), vec![]).is_err()
        );
    }

    #[test]
    fn decodes_required_token_actions_for_both_programs() {
        for program in [TOKEN_PROGRAM, TOKEN_2022_PROGRAM] {
            for tag in [
                0_u8, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 20, 22,
            ] {
                let mut data = vec![tag];
                data.extend_from_slice(&[0; 40]);
                assert!(
                    decode_instruction(&instruction(program, data), vec![]).is_ok(),
                    "tag {tag}"
                );
            }
        }
    }

    #[test]
    fn decodes_all_required_builtin_discriminants() {
        for tag in 0_u32..=12 {
            let mut data = tag.to_le_bytes().to_vec();
            if tag == 7 {
                data.extend_from_slice(&[0; 32]);
            }
            assert!(decode_instruction(&instruction(SYSTEM_PROGRAM, data), vec![]).is_ok());
        }
        for tag in 0_u32..=4 {
            assert!(
                decode_instruction(
                    &instruction(ALT_PROGRAM, tag.to_le_bytes().to_vec()),
                    vec![]
                )
                .is_ok()
            );
        }
        for tag in 0_u32..=7 {
            assert!(
                decode_instruction(
                    &instruction(UPGRADEABLE_LOADER, tag.to_le_bytes().to_vec()),
                    vec![]
                )
                .is_ok()
            );
        }
        assert!(
            decode_instruction(&instruction(ASSOCIATED_TOKEN_PROGRAM, vec![1]), vec![]).is_ok()
        );
        assert!(
            decode_instruction(&instruction(MEMO_PROGRAM, b"guardian".to_vec()), vec![]).is_ok()
        );
    }

    #[test]
    fn preserves_unknown_program_evidence() {
        let unknown = instruction("Vote111111111111111111111111111111111111111", vec![1, 2, 3]);
        let action = decode_instruction(&unknown, vec![]).unwrap_or_else(|_| unreachable!());
        assert!(!action.known);
        assert!(action.details.contains_key("data_sha256"));
    }

    #[test]
    fn identifies_priority_token_2022_extensions() {
        let mut data = Vec::new();
        data.extend_from_slice(&1_u16.to_le_bytes());
        data.extend_from_slice(&2_u16.to_le_bytes());
        data.extend_from_slice(&[0, 0]);
        data.extend_from_slice(&14_u16.to_le_bytes());
        data.extend_from_slice(&0_u16.to_le_bytes());
        assert_eq!(
            token_2022_extension_names(&data, 0),
            Ok(vec![
                "transfer_fee_config".to_owned(),
                "transfer_hook".to_owned()
            ])
        );
    }

    #[test]
    fn rejects_malformed_or_duplicate_token_2022_tlv() {
        assert!(token_2022_extension_names(&[1, 0, 4, 0, 0], 0).is_err());

        let mut duplicate = Vec::new();
        for _ in 0..2 {
            duplicate.extend_from_slice(&14_u16.to_le_bytes());
            duplicate.extend_from_slice(&0_u16.to_le_bytes());
        }
        assert!(token_2022_extension_names(&duplicate, 0).is_err());
    }
}
