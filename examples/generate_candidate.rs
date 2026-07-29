use std::env;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use solana_message::{
    Address, Message, MessageHeader, VersionedMessage, compiled_instruction::CompiledInstruction,
    v0,
};
use solana_transaction::{Signature, versioned::VersionedTransaction};

fn system_transfer_data(lamports: u64) -> Vec<u8> {
    let mut data = 2_u32.to_le_bytes().to_vec();
    data.extend_from_slice(&lamports.to_le_bytes());
    data
}

fn simple_transfer() -> Result<VersionedTransaction, Box<dyn std::error::Error>> {
    Ok(VersionedTransaction {
        signatures: vec![Signature::default()],
        message: VersionedMessage::Legacy(Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![
                Address::new_from_array([1; 32]),
                Address::new_from_array([2; 32]),
                "11111111111111111111111111111111".parse()?,
            ],
            recent_blockhash: solana_message::Hash::default(),
            instructions: vec![CompiledInstruction {
                program_id_index: 2,
                accounts: vec![0, 1],
                data: system_transfer_data(1),
            }],
        }),
    })
}

fn transfer_with_delegate() -> Result<VersionedTransaction, Box<dyn std::error::Error>> {
    let mut approve_data = vec![4];
    approve_data.extend_from_slice(&1_000_000_u64.to_le_bytes());
    Ok(VersionedTransaction {
        signatures: vec![Signature::default()],
        message: VersionedMessage::Legacy(Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 3,
            },
            account_keys: vec![
                Address::new_from_array([1; 32]),
                Address::new_from_array([2; 32]),
                Address::new_from_array([3; 32]),
                Address::new_from_array([4; 32]),
                "11111111111111111111111111111111".parse()?,
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse()?,
            ],
            recent_blockhash: solana_message::Hash::default(),
            instructions: vec![
                CompiledInstruction {
                    program_id_index: 4,
                    accounts: vec![0, 1],
                    data: system_transfer_data(1),
                },
                CompiledInstruction {
                    program_id_index: 5,
                    accounts: vec![2, 3, 0],
                    data: approve_data,
                },
            ],
        }),
    })
}

fn unknown_program() -> VersionedTransaction {
    VersionedTransaction {
        signatures: vec![Signature::default()],
        message: VersionedMessage::Legacy(Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![
                Address::new_from_array([1; 32]),
                Address::new_from_array([9; 32]),
            ],
            recent_blockhash: solana_message::Hash::default(),
            instructions: vec![CompiledInstruction {
                program_id_index: 1,
                accounts: vec![0],
                data: vec![0xde, 0xad, 0xbe, 0xef],
            }],
        }),
    }
}

fn v0_with_alt() -> Result<VersionedTransaction, Box<dyn std::error::Error>> {
    Ok(VersionedTransaction {
        signatures: vec![Signature::default()],
        message: VersionedMessage::V0(v0::Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![
                Address::new_from_array([1; 32]),
                "11111111111111111111111111111111".parse()?,
            ],
            recent_blockhash: solana_message::Hash::default(),
            instructions: vec![CompiledInstruction {
                program_id_index: 1,
                accounts: vec![0, 2],
                data: system_transfer_data(1),
            }],
            address_table_lookups: vec![v0::MessageAddressTableLookup {
                account_key: "9DswaXsjcqozpbUUnL24wRqteqZTZH1UqCpFcsYWcgQP".parse()?,
                writable_indexes: vec![23],
                readonly_indexes: vec![],
            }],
        }),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = env::args().nth(1).unwrap_or_else(|| "simple".to_owned());
    let transaction = match fixture.as_str() {
        "simple" => simple_transfer()?,
        "delegate" => transfer_with_delegate()?,
        "unknown" => unknown_program(),
        "v0-alt" => v0_with_alt()?,
        _ => return Err("fixture must be simple, delegate, unknown, or v0-alt".into()),
    };
    println!("{}", STANDARD.encode(bincode::serialize(&transaction)?));
    Ok(())
}
