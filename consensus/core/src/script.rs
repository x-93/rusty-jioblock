use std::io::Cursor;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::{
    errors::ConsensusError,
    hashing::{self, Hash},
};

/// Script opcodes
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    // Constants
    OP_0 = 0x00,
    OP_1 = 0x51,
    OP_2 = 0x52,
    OP_3 = 0x53,
    OP_4 = 0x54,
    OP_5 = 0x55,
    OP_6 = 0x56,
    OP_7 = 0x57,
    OP_8 = 0x58,
    OP_9 = 0x59,
    OP_10 = 0x5a,
    OP_11 = 0x5b,
    OP_12 = 0x5c,
    OP_13 = 0x5d,
    OP_14 = 0x5e,
    OP_15 = 0x5f,
    OP_16 = 0x60,

    // Flow control
    OP_NOP = 0x61,
    OP_IF = 0x63,
    OP_ELSE = 0x67,
    OP_ENDIF = 0x68,
    OP_VERIFY = 0x69,
    OP_RETURN = 0x6a,

    // Stack
    OP_DUP = 0x76,
    OP_SWAP = 0x7c,
    OP_2DUP = 0x6e,
    OP_3DUP = 0x6f,
    OP_2SWAP = 0x72,
    OP_IFDUP = 0x73,
    OP_DEPTH = 0x74,
    OP_DROP = 0x75,
    OP_NIP = 0x77,
    OP_OVER = 0x78,
    OP_PICK = 0x79,
    OP_ROLL = 0x7a,
    OP_ROT = 0x7b,
    OP_TUCK = 0x7d,
    OP_2DROP = 0x6d,
    OP_2OVER = 0x70,
    OP_2ROT = 0x71,

    // Splice
    OP_SIZE = 0x82,

    // Bitwise logic
    OP_EQUAL = 0x87,
    OP_EQUALVERIFY = 0x88,

    // Arithmetic
    OP_1ADD = 0x8b,
    OP_1SUB = 0x8c,
    OP_NEGATE = 0x8f,
    OP_ABS = 0x90,
    OP_NOT = 0x91,
    OP_0NOTEQUAL = 0x92,
    OP_ADD = 0x93,
    OP_SUB = 0x94,
    OP_MUL = 0x95,
    OP_DIV = 0x96,
    OP_MOD = 0x97,
    OP_LSHIFT = 0x98,
    OP_RSHIFT = 0x99,

    // Crypto
    OP_SHA256 = 0xa8,
    OP_HASH160 = 0xa9,
    OP_HASH256 = 0xaa,
    OP_CODESEPARATOR = 0xab,
    OP_CHECKSIG = 0xac,
    OP_CHECKSIGVERIFY = 0xad,
    OP_CHECKMULTISIG = 0xae,
    OP_CHECKMULTISIGVERIFY = 0xaf,

    // Reserved words
    OP_RESERVED = 0x50,
    OP_VER = 0x62,
    OP_VERIF = 0x65,
    OP_VERNOTIF = 0x66,
    OP_RESERVED1 = 0x89,
    OP_RESERVED2 = 0x8a,
}

/// Represents a script containing opcodes and data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Script {
    /// Raw script bytes
    bytes: Vec<u8>,
}

impl Script {
    /// Creates a new empty script
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Creates a script from raw bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Returns the raw script bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Creates a P2PKH script pubkey
    pub fn p2pkh_script_pubkey(pubkey_hash: &[u8; 20]) -> Self {
        let mut script = Vec::with_capacity(25);
        script.push(Opcode::OP_DUP as u8);
        script.push(Opcode::OP_HASH160 as u8);
        script.push(20); // Length of pubkey hash
        script.extend_from_slice(pubkey_hash);
        script.push(Opcode::OP_EQUALVERIFY as u8);
        script.push(Opcode::OP_CHECKSIG as u8);
        Self { bytes: script }
    }

    /// Creates a P2PKH signature script
    pub fn p2pkh_signature_script(signature: &[u8], pubkey: &[u8]) -> Self {
        let mut script = Vec::new();
        script.push(signature.len() as u8);
        script.extend_from_slice(signature);
        script.push(pubkey.len() as u8);
        script.extend_from_slice(pubkey);
        Self { bytes: script }
    }

    /// Validates the script structure
    pub fn validate(&self) -> Result<(), ConsensusError> {
        if self.bytes.is_empty() {
            return Err(ConsensusError::InvalidScript);
        }

        let mut cursor = Cursor::new(&self.bytes);
        while cursor.position() < self.bytes.len() as u64 {
            let opcode = cursor.get_ref()[cursor.position() as usize];
            cursor.set_position(cursor.position() + 1);

            match opcode {
                // Push data operations
                0x01..=0x4b => {
                    let len = opcode as usize;
                    if cursor.position() as usize + len > self.bytes.len() {
                        return Err(ConsensusError::InvalidScript);
                    }
                    cursor.set_position(cursor.position() + len as u64);
                }
                // Other opcodes
                _ => {
                    if !Self::is_valid_opcode(opcode) {
                        return Err(ConsensusError::InvalidScript);
                    }
                }
            }
        }

        Ok(())
    }

    /// Returns true if the opcode is valid
    fn is_valid_opcode(opcode: u8) -> bool {
        matches!(opcode, 
            0x00 | // OP_0
            0x51..=0x60 | // OP_1 to OP_16
            0x61 | // OP_NOP
            0x63 | // OP_IF
            0x67 | // OP_ELSE
            0x68 | // OP_ENDIF
            0x69 | // OP_VERIFY
            0x6a | // OP_RETURN
            0x76 | // OP_DUP
            0x87 | // OP_EQUAL
            0x88 | // OP_EQUALVERIFY
            0xa8 | // OP_SHA256
            0xa9 | // OP_HASH160
            0xaa | // OP_HASH256
            0xac | // OP_CHECKSIG
            0xad | // OP_CHECKSIGVERIFY
            0xae | // OP_CHECKMULTISIG
            0xaf   // OP_CHECKMULTISIGVERIFY
        )
    }
}