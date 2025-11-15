use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum SigHashType {
    SighashAll = 1,
    SighashNone = 2,
    SighashSingle = 3,
    SighashAnyoneCanPay = 0x80,
}

impl Default for SigHashType {
    fn default() -> Self {
        SigHashType::SighashAll
    }
}