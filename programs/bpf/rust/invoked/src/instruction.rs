//! @brief Example Rust-based BPF program that issues a cross-program-invocation

use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

pub const VERIFY_TRANSLATIONS: u8 = 0;
pub const RETURN_ERROR: u8 = 1;
pub const DERIVED_SIGNERS: u8 = 2;
pub const VERIFY_NESTED_SIGNERS: u8 = 3;
pub const VERIFY_WRITER: u8 = 4;
pub const VERIFY_PRIVILEGE_ESCALATION: u8 = 5;
pub const NESTED_INVOKE: u8 = 6;
pub const RETURN_OK: u8 = 7;
pub const VERIFY_PRIVILEGE_DEESCALATION: u8 = 8;
pub const VERIFY_PRIVILEGE_DEESCALATION_ESCALATION_SIGNER: u8 = 9;
pub const VERIFY_PRIVILEGE_DEESCALATION_ESCALATION_WRITABLE: u8 = 10;
pub const WRITE_ACCOUNT: u8 = 11;

pub fn create_instruction(
    program_id: Pubkey,
    arguments: &[(&Pubkey, bool, bool)],
    data: Vec<u8>,
) -> Instruction {
    let accounts = arguments
        .iter()
        .map(|(key, is_writable, is_signer)| {
            if *is_writable {
                AccountMeta::new(**key, *is_signer)
            } else {
                AccountMeta::new_readonly(**key, *is_signer)
            }
        })
        .collect();
    Instruction {
        program_id,
        accounts,
        data,
    }
}