//! M0 dependency compatibility probes retained as regression coverage.

use solana_message::VersionedMessage;
use solana_transaction::versioned::VersionedTransaction;
use thiserror::Error;

/// Errors produced while decoding a wire transaction during the M0 spike.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpikeError {
    #[error("transaction wire data is invalid")]
    Deserialize,
    #[error("transaction version is not supported")]
    UnsupportedVersion,
    #[error("transaction structure is invalid")]
    InvalidStructure,
}

/// Decode and structurally validate a legacy or v0 Solana transaction.
///
/// # Errors
///
/// Returns a controlled error when the wire format is malformed, the message
/// version is newer than v0, or the transaction fails structural validation.
pub fn decode_supported_transaction(bytes: &[u8]) -> Result<VersionedTransaction, SpikeError> {
    let transaction: VersionedTransaction =
        bincode::deserialize(bytes).map_err(|_| SpikeError::Deserialize)?;

    match &transaction.message {
        VersionedMessage::Legacy(_) | VersionedMessage::V0(_) => {}
        VersionedMessage::V1(_) => return Err(SpikeError::UnsupportedVersion),
    }

    transaction
        .message
        .sanitize()
        .map_err(|_| SpikeError::InvalidStructure)?;

    if transaction.signatures.len()
        != usize::from(transaction.message.header().num_required_signatures)
    {
        return Err(SpikeError::InvalidStructure);
    }

    Ok(transaction)
}

#[cfg(test)]
mod tests {
    use solana_message::{
        Message, MessageHeader, VersionedMessage, compiled_instruction::CompiledInstruction, v0,
    };
    use solana_transaction::versioned::VersionedTransaction;

    use super::{SpikeError, decode_supported_transaction};

    fn header() -> MessageHeader {
        MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 1,
        }
    }

    #[test]
    fn decodes_legacy_wire_transaction() {
        let transaction = VersionedTransaction {
            signatures: vec![solana_transaction::Signature::default()],
            message: VersionedMessage::Legacy(Message {
                header: header(),
                account_keys: vec![
                    solana_message::Address::new_from_array([1; 32]),
                    solana_message::Address::new_from_array([2; 32]),
                ],
                recent_blockhash: solana_message::Hash::default(),
                instructions: vec![CompiledInstruction {
                    program_id_index: 1,
                    accounts: vec![],
                    data: vec![],
                }],
            }),
        };
        let wire = bincode::serialize(&transaction).unwrap_or_default();

        let decoded = decode_supported_transaction(&wire);

        assert!(matches!(
            decoded,
            Ok(VersionedTransaction {
                message: VersionedMessage::Legacy(_),
                ..
            })
        ));
    }

    #[test]
    fn decodes_v0_wire_transaction() {
        let transaction = VersionedTransaction {
            signatures: vec![solana_transaction::Signature::default()],
            message: VersionedMessage::V0(v0::Message {
                header: header(),
                account_keys: vec![
                    solana_message::Address::new_from_array([1; 32]),
                    solana_message::Address::new_from_array([2; 32]),
                ],
                recent_blockhash: solana_message::Hash::default(),
                instructions: vec![CompiledInstruction {
                    program_id_index: 1,
                    accounts: vec![],
                    data: vec![],
                }],
                address_table_lookups: vec![],
            }),
        };
        let wire = bincode::serialize(&transaction).unwrap_or_default();

        let decoded = decode_supported_transaction(&wire);

        assert!(matches!(
            decoded,
            Ok(VersionedTransaction {
                message: VersionedMessage::V0(_),
                ..
            })
        ));
    }

    #[test]
    fn rejects_malformed_wire_transaction() {
        assert_eq!(
            decode_supported_transaction(&[0xff, 0xff]),
            Err(SpikeError::Deserialize)
        );
    }

    #[test]
    fn token_interfaces_unpack_transfer() {
        let mut transfer = vec![3];
        transfer.extend_from_slice(&42_u64.to_le_bytes());

        let legacy = spl_token_interface::instruction::TokenInstruction::unpack(&transfer);
        let token_2022 = spl_token_2022_interface::instruction::TokenInstruction::unpack(&transfer);

        assert!(legacy.is_ok());
        assert!(token_2022.is_ok());
    }
}
