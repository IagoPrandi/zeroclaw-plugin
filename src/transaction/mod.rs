use std::collections::HashMap;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};
use solana_message::{VersionedMessage, compiled_instruction::CompiledInstruction};
use solana_transaction::versioned::VersionedTransaction;

use crate::{address::Address32, error::GuardianError, limits::Limits};

const ALT_META_SIZE: usize = 56;
const ALT_STATE_LOOKUP_TABLE: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionVersion {
    Legacy,
    V0,
}

impl TransactionVersion {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::V0 => "v0",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountKeySource {
    Static,
    LookupWritable,
    LookupReadonly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAccountKey {
    pub address: Address32,
    pub source: AccountKeySource,
    pub signer: bool,
    pub writable: bool,
    pub fee_payer: bool,
    pub executable: Option<bool>,
    pub owner: Option<Address32>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedInstruction {
    pub instruction_index: usize,
    pub program_id: Address32,
    pub program_id_index: usize,
    pub accounts: Vec<usize>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressTableLookup {
    pub table: Address32,
    pub writable_indexes: Vec<u8>,
    pub readonly_indexes: Vec<u8>,
    pub last_extended_slot: u64,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedTransaction {
    pub version: TransactionVersion,
    pub signatures: Vec<[u8; 64]>,
    pub message_hash: [u8; 32],
    pub recent_blockhash: [u8; 32],
    pub fee_payer: Address32,
    pub account_keys: Vec<ResolvedAccountKey>,
    pub instructions: Vec<ResolvedInstruction>,
    pub address_table_lookups: Vec<AddressTableLookup>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedLookupTable {
    pub addresses: Vec<Address32>,
    pub last_extended_slot: u64,
    pub active: bool,
}

/// Decode an address lookup table account payload.
///
/// The caller must separately verify that the account owner is the Address
/// Lookup Table program.
///
/// # Errors
///
/// Returns `AddressLookupTable` for a wrong state discriminant, truncated
/// metadata, or a non-integral address vector.
pub fn decode_lookup_table(data: &[u8]) -> Result<DecodedLookupTable, GuardianError> {
    if data.len() < ALT_META_SIZE || !(data.len() - ALT_META_SIZE).is_multiple_of(32) {
        return Err(GuardianError::AddressLookupTable);
    }
    let state = u32::from_le_bytes(
        data.get(0..4)
            .and_then(|value| value.try_into().ok())
            .ok_or(GuardianError::AddressLookupTable)?,
    );
    if state != ALT_STATE_LOOKUP_TABLE {
        return Err(GuardianError::AddressLookupTable);
    }
    let deactivation_slot = u64::from_le_bytes(
        data.get(4..12)
            .and_then(|value| value.try_into().ok())
            .ok_or(GuardianError::AddressLookupTable)?,
    );
    let last_extended_slot = u64::from_le_bytes(
        data.get(12..20)
            .and_then(|value| value.try_into().ok())
            .ok_or(GuardianError::AddressLookupTable)?,
    );
    let addresses = data[ALT_META_SIZE..]
        .chunks_exact(32)
        .map(|chunk| {
            <[u8; 32]>::try_from(chunk)
                .map(Address32::new)
                .map_err(|_| GuardianError::AddressLookupTable)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DecodedLookupTable {
        addresses,
        last_extended_slot,
        active: deactivation_slot == u64::MAX,
    })
}

/// Decode and fully normalize a serialized transaction.
///
/// # Errors
///
/// Returns a controlled error for Base64, size, wire-structure, future-version,
/// unresolved lookup, inactive lookup, or index failures.
#[allow(clippy::implicit_hasher)]
pub fn normalize_base64(
    encoded: &str,
    limits: &Limits,
    lookup_tables: &HashMap<Address32, DecodedLookupTable>,
) -> Result<NormalizedTransaction, GuardianError> {
    let maximum_encoded = limits
        .max_transaction_bytes
        .checked_add(2)
        .and_then(|value| value.checked_div(3))
        .and_then(|value| value.checked_mul(4))
        .ok_or(GuardianError::ArithmeticOverflow)?;
    if encoded.len() > maximum_encoded {
        return Err(GuardianError::TransactionTooLarge);
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| GuardianError::Base64Decode)?;
    if bytes.len() > limits.max_transaction_bytes {
        return Err(GuardianError::TransactionTooLarge);
    }
    normalize_wire(&bytes, limits, lookup_tables)
}

/// Decode and normalize transaction wire bytes.
///
/// # Errors
///
/// Returns a controlled validation error and never panics for arbitrary bytes.
#[allow(clippy::implicit_hasher)]
pub fn normalize_wire(
    bytes: &[u8],
    limits: &Limits,
    lookup_tables: &HashMap<Address32, DecodedLookupTable>,
) -> Result<NormalizedTransaction, GuardianError> {
    if bytes.len() > limits.max_transaction_bytes {
        return Err(GuardianError::TransactionTooLarge);
    }
    let transaction: VersionedTransaction =
        bincode::deserialize(bytes).map_err(|_| GuardianError::TransactionDeserialize)?;
    transaction
        .message
        .sanitize()
        .map_err(|_| GuardianError::TransactionDeserialize)?;
    if transaction.signatures.len()
        != usize::from(transaction.message.header().num_required_signatures)
    {
        return Err(GuardianError::TransactionDeserialize);
    }
    if matches!(transaction.message, VersionedMessage::V1(_)) {
        return Err(GuardianError::UnsupportedVersion);
    }

    let signatures = transaction
        .signatures
        .iter()
        .map(|signature| *signature.as_array())
        .collect();
    let serialized_message =
        bincode::serialize(&transaction.message).map_err(|_| GuardianError::Internal)?;
    let message_hash: [u8; 32] = Sha256::digest(serialized_message).into();
    let recent_blockhash = match &transaction.message {
        VersionedMessage::Legacy(message) => message.recent_blockhash.to_bytes(),
        VersionedMessage::V0(message) => message.recent_blockhash.to_bytes(),
        VersionedMessage::V1(_) => return Err(GuardianError::UnsupportedVersion),
    };

    let mut account_keys = static_account_keys(&transaction.message);
    let mut address_table_lookups = Vec::new();
    if let VersionedMessage::V0(message) = &transaction.message {
        resolve_lookups(
            message,
            lookup_tables,
            &mut account_keys,
            &mut address_table_lookups,
        )?;
    }
    if account_keys.is_empty() || account_keys.len() > limits.max_accounts {
        return Err(GuardianError::InvalidAccountIndex);
    }
    if transaction.message.instructions().len() > limits.max_instructions {
        return Err(GuardianError::InvalidAccountIndex);
    }
    let instructions = resolve_instructions(transaction.message.instructions(), &account_keys)?;
    let fee_payer = account_keys
        .first()
        .map(|key| key.address)
        .ok_or(GuardianError::InvalidAccountIndex)?;

    Ok(NormalizedTransaction {
        version: match transaction.message {
            VersionedMessage::Legacy(_) => TransactionVersion::Legacy,
            VersionedMessage::V0(_) => TransactionVersion::V0,
            VersionedMessage::V1(_) => return Err(GuardianError::UnsupportedVersion),
        },
        signatures,
        message_hash,
        recent_blockhash,
        fee_payer,
        account_keys,
        instructions,
        address_table_lookups,
    })
}

fn static_account_keys(message: &VersionedMessage) -> Vec<ResolvedAccountKey> {
    message
        .static_account_keys()
        .iter()
        .enumerate()
        .map(|(index, address)| ResolvedAccountKey {
            address: Address32::new(address.to_bytes()),
            source: AccountKeySource::Static,
            signer: message.is_signer(index),
            writable: message.is_maybe_writable_with_reserved_addresses(
                index,
                None::<&std::collections::HashSet<solana_message::Address>>,
            ),
            fee_payer: index == 0,
            executable: None,
            owner: None,
            label: None,
        })
        .collect()
}

fn resolve_lookups(
    message: &solana_message::v0::Message,
    tables: &HashMap<Address32, DecodedLookupTable>,
    keys: &mut Vec<ResolvedAccountKey>,
    output: &mut Vec<AddressTableLookup>,
) -> Result<(), GuardianError> {
    for lookup in &message.address_table_lookups {
        let table_address = Address32::new(lookup.account_key.to_bytes());
        let table = tables
            .get(&table_address)
            .ok_or(GuardianError::AddressLookupTable)?;
        if !table.active {
            return Err(GuardianError::AddressLookupTable);
        }
        for (indexes, source, writable) in [
            (
                lookup.writable_indexes.as_slice(),
                AccountKeySource::LookupWritable,
                true,
            ),
            (
                lookup.readonly_indexes.as_slice(),
                AccountKeySource::LookupReadonly,
                false,
            ),
        ] {
            for index in indexes {
                let address = table
                    .addresses
                    .get(usize::from(*index))
                    .copied()
                    .ok_or(GuardianError::AddressLookupTable)?;
                keys.push(ResolvedAccountKey {
                    address,
                    source,
                    signer: false,
                    writable,
                    fee_payer: false,
                    executable: None,
                    owner: None,
                    label: None,
                });
            }
        }
        output.push(AddressTableLookup {
            table: table_address,
            writable_indexes: lookup.writable_indexes.clone(),
            readonly_indexes: lookup.readonly_indexes.clone(),
            last_extended_slot: table.last_extended_slot,
            active: table.active,
        });
    }
    Ok(())
}

fn resolve_instructions(
    instructions: &[CompiledInstruction],
    keys: &[ResolvedAccountKey],
) -> Result<Vec<ResolvedInstruction>, GuardianError> {
    instructions
        .iter()
        .enumerate()
        .map(|(instruction_index, instruction)| {
            let program_id_index = usize::from(instruction.program_id_index);
            let program_id = keys
                .get(program_id_index)
                .map(|key| key.address)
                .ok_or(GuardianError::InvalidAccountIndex)?;
            let accounts = instruction
                .accounts
                .iter()
                .map(|index| {
                    let index = usize::from(*index);
                    keys.get(index)
                        .map(|_| index)
                        .ok_or(GuardianError::InvalidAccountIndex)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ResolvedInstruction {
                instruction_index,
                program_id,
                program_id_index,
                accounts,
                data: instruction.data.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use proptest::prelude::*;
    use solana_message::{
        Message, MessageHeader, VersionedMessage, compiled_instruction::CompiledInstruction, v0,
    };
    use solana_transaction::versioned::VersionedTransaction;

    use super::{
        ALT_META_SIZE, AccountKeySource, DecodedLookupTable, TransactionVersion,
        decode_lookup_table, normalize_wire,
    };
    use crate::{address::Address32, limits::Limits};

    fn limits() -> Limits {
        Limits {
            max_rpc_calls: 8,
            max_http_response_bytes: 2048,
            max_transaction_bytes: 1232,
            max_output_bytes: 8192,
            max_accounts: 512,
            max_instructions: 256,
        }
    }

    fn header() -> MessageHeader {
        MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 1,
        }
    }

    fn wire(message: VersionedMessage) -> Vec<u8> {
        bincode::serialize(&VersionedTransaction {
            signatures: vec![solana_transaction::Signature::default()],
            message,
        })
        .unwrap_or_default()
    }

    #[test]
    fn normalizes_equivalent_legacy_and_v0() {
        let accounts = vec![
            solana_message::Address::new_from_array([1; 32]),
            solana_message::Address::new_from_array([2; 32]),
        ];
        let instruction = CompiledInstruction {
            program_id_index: 1,
            accounts: vec![0],
            data: vec![9],
        };
        let legacy = normalize_wire(
            &wire(VersionedMessage::Legacy(Message {
                header: header(),
                account_keys: accounts.clone(),
                recent_blockhash: solana_message::Hash::default(),
                instructions: vec![instruction.clone()],
            })),
            &limits(),
            &HashMap::new(),
        );
        let versioned = normalize_wire(
            &wire(VersionedMessage::V0(v0::Message {
                header: header(),
                account_keys: accounts,
                recent_blockhash: solana_message::Hash::default(),
                instructions: vec![instruction],
                address_table_lookups: vec![],
            })),
            &limits(),
            &HashMap::new(),
        );
        assert!(legacy.is_ok() && versioned.is_ok());
        let (legacy, versioned) = (
            legacy.unwrap_or_else(|_| unreachable!()),
            versioned.unwrap_or_else(|_| unreachable!()),
        );
        assert_eq!(legacy.version, TransactionVersion::Legacy);
        assert_eq!(versioned.version, TransactionVersion::V0);
        assert_eq!(legacy.account_keys, versioned.account_keys);
        assert_eq!(legacy.instructions, versioned.instructions);
    }

    #[test]
    fn resolves_lookup_order_and_rejects_missing_or_bad_index() {
        let table_key = solana_message::Address::new_from_array([3; 32]);
        let message = VersionedMessage::V0(v0::Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![
                solana_message::Address::new_from_array([1; 32]),
                solana_message::Address::new_from_array([2; 32]),
            ],
            recent_blockhash: solana_message::Hash::default(),
            instructions: vec![CompiledInstruction {
                program_id_index: 1,
                accounts: vec![2],
                data: vec![],
            }],
            address_table_lookups: vec![v0::MessageAddressTableLookup {
                account_key: table_key,
                writable_indexes: vec![1],
                readonly_indexes: vec![0],
            }],
        });
        assert!(normalize_wire(&wire(message.clone()), &limits(), &HashMap::new()).is_err());
        let mut tables = HashMap::new();
        tables.insert(
            Address32::new(table_key.to_bytes()),
            DecodedLookupTable {
                addresses: vec![Address32::new([4; 32]), Address32::new([5; 32])],
                last_extended_slot: 7,
                active: true,
            },
        );
        let normalized = normalize_wire(&wire(message), &limits(), &tables);
        assert!(normalized.is_ok(), "{normalized:?}");
        let normalized = normalized.unwrap_or_else(|_| unreachable!());
        assert_eq!(
            normalized.account_keys[2].source,
            AccountKeySource::LookupWritable
        );
        assert_eq!(
            normalized.account_keys[3].source,
            AccountKeySource::LookupReadonly
        );
        assert_eq!(
            normalized.instructions[0].program_id,
            Address32::new([2; 32])
        );
    }

    #[test]
    fn decodes_active_lookup_table() {
        let mut data = vec![0; ALT_META_SIZE];
        data[0..4].copy_from_slice(&1_u32.to_le_bytes());
        data[4..12].copy_from_slice(&u64::MAX.to_le_bytes());
        data[12..20].copy_from_slice(&42_u64.to_le_bytes());
        data.extend_from_slice(&[8; 32]);
        let table = decode_lookup_table(&data);
        assert_eq!(
            table,
            Ok(DecodedLookupTable {
                addresses: vec![Address32::new([8; 32])],
                last_extended_slot: 42,
                active: true
            })
        );
    }

    proptest! {
        #[test]
        fn arbitrary_wire_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
            let _ = normalize_wire(&bytes, &limits(), &HashMap::new());
        }
    }
}
