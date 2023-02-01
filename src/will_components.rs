use secp256k1::{SecretKey, PublicKey};
use crate::traits::*;

#[derive(Debug, Clone)]
pub struct TimelockComponents {
    pub single_use_private_key: SecretKey,
    pub single_use_public_key: PublicKey,
    pub sequence_locktime: [u8;2],
    pub sequence_flags: [u8;2],
    pub witness_script: Vec<u8>,
    pub locking_script: Vec<u8>,
}

    impl TimelockComponents{
        pub fn new(single_use_private_key: SecretKey, single_use_public_key: PublicKey, locktime: u16, witness_script: Vec<u8>, locking_script: Vec<u8>) -> Result<Self, String>{
            Ok(TimelockComponents{
                single_use_private_key,
                single_use_public_key,
                sequence_locktime: locktime.to_le_bytes(),
                sequence_flags: [0u8;2],
                witness_script,
                locking_script,
            })
        }

        pub fn sequence(&self) -> Vec<u8> {
            let mut result: Vec<u8> = Vec::new();
            result.extend_from_slice(&self.sequence_locktime);
            result.extend_from_slice(&self.sequence_flags);
            result
        }
    }