use base64::{Engine as _, engine::general_purpose::STANDARD};
use solana_message::{
    Message, MessageHeader, VersionedMessage, compiled_instruction::CompiledInstruction,
};
use solana_transaction::versioned::VersionedTransaction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut transfer_data = 2_u32.to_le_bytes().to_vec();
    transfer_data.extend_from_slice(&1_u64.to_le_bytes());
    let transaction = VersionedTransaction {
        signatures: vec![solana_transaction::Signature::default()],
        message: VersionedMessage::Legacy(Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![
                solana_message::Address::new_from_array([1; 32]),
                solana_message::Address::new_from_array([2; 32]),
                "11111111111111111111111111111111".parse()?,
            ],
            recent_blockhash: solana_message::Hash::default(),
            instructions: vec![CompiledInstruction {
                program_id_index: 2,
                accounts: vec![0, 1],
                data: transfer_data,
            }],
        }),
    };
    println!("{}", STANDARD.encode(bincode::serialize(&transaction)?));
    Ok(())
}
