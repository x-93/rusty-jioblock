//!
//! # Transaction
//!
//! This module implements consensus [`Transaction`] structure and related types.
//!

#![allow(non_snake_case)]

mod script_public_key;

use borsh::{BorshDeserialize, BorshSerialize};
use jio_utils::hex::ToHex;
use jio_utils::mem_size::MemSizeEstimator;
use jio_utils::{serde_bytes, serde_bytes_fixed_ref};
pub use script_public_key::{
    scriptvec, ScriptPublicKey, ScriptPublicKeyT, ScriptPublicKeyVersion, ScriptPublicKeys, ScriptVec, SCRIPT_VECTOR_SIZE,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;
use std::{
    fmt::Display,
    ops::Range,
    str::{self},
};
use wasm_bindgen::prelude::*;

use crate::mass::{ContextualMasses, NonContextualMasses};
use crate::{
    hashing,
    subnets::{self, SubnetworkId},
};
use crate::errors::ConsensusError;
use crate::Hash;

/// COINBASE_TRANSACTION_INDEX is the index of the coinbase transaction in every block
pub const COINBASE_TRANSACTION_INDEX: usize = 0;
/// A 32-byte Jio transaction identifier.
pub type TransactionId = crate::Hash;

/// Holds details about an individual transaction output in a utxo
/// set such as whether or not it was contained in a coinbase tx, the daa
/// score of the block that accepts the tx, its public key script, and how
/// much it pays.
/// @category Consensus
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(inspectable, js_name = TransactionUtxoEntry)]
pub struct UtxoEntry {
    pub amount: u64,
    #[wasm_bindgen(js_name = scriptPublicKey, getter_with_clone)]
    pub script_public_key: ScriptPublicKey,
    #[wasm_bindgen(js_name = blockDaaScore)]
    pub block_daa_score: u64,
    #[wasm_bindgen(js_name = isCoinbase)]
    pub is_coinbase: bool,
}

impl UtxoEntry {
    pub fn new(amount: u64, script_public_key: ScriptPublicKey, block_daa_score: u64, is_coinbase: bool) -> Self {
        Self { amount, script_public_key, block_daa_score, is_coinbase }
    }
}

impl MemSizeEstimator for UtxoEntry {}

pub type TransactionIndexType = u32;

/// Represents a Jio transaction outpoint
#[derive(Eq, Default, Hash, PartialEq, Debug, Copy, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOutpoint {
    #[serde(with = "serde_bytes_fixed_ref")]
    pub transaction_id: TransactionId,
    pub index: TransactionIndexType,
}

impl TransactionOutpoint {
    pub fn new(transaction_id: TransactionId, index: u32) -> Self {
        Self { transaction_id, index }
    }
}

impl Display for TransactionOutpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.transaction_id, self.index)
    }
}

/// Represents a Jio transaction input
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInput {
    pub previous_outpoint: TransactionOutpoint,
    #[serde(with = "serde_bytes")]
    pub signature_script: Vec<u8>, // TODO: Consider using SmallVec
    pub sequence: u64,

    // TODO: Since this field is used for calculating mass context free, and we already commit
    // to the mass in a dedicated field (on the tx level), it follows that this field is no longer
    // needed, and can be removed if we ever implement a v2 transaction
    pub sig_op_count: u8,
}

impl TransactionInput {
    pub fn new(previous_outpoint: TransactionOutpoint, signature_script: Vec<u8>, sequence: u64, sig_op_count: u8) -> Self {
        Self { previous_outpoint, signature_script, sequence, sig_op_count }
    }
}

impl std::fmt::Debug for TransactionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionInput")
            .field("previous_outpoint", &self.previous_outpoint)
            .field("signature_script", &self.signature_script.to_hex())
            .field("sequence", &self.sequence)
            .field("sig_op_count", &self.sig_op_count)
            .finish()
    }
}

/// Represents a Jiopad transaction output
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOutput {
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
}

impl TransactionOutput {
    pub fn new(value: u64, script_public_key: ScriptPublicKey) -> Self {
        Self { value, script_public_key }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransactionMass(AtomicU64); // TODO: using atomic as a temp solution for mutating this field through the mempool

impl Eq for TransactionMass {}

impl PartialEq for TransactionMass {
    fn eq(&self, other: &Self) -> bool {
        self.0.load(SeqCst) == other.0.load(SeqCst)
    }
}

impl Clone for TransactionMass {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(SeqCst)))
    }
}

impl BorshDeserialize for TransactionMass {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let mass: u64 = BorshDeserialize::deserialize(buf)?;
        Ok(Self(AtomicU64::new(mass)))
    }
}

impl BorshSerialize for TransactionMass {
    fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        borsh::BorshSerialize::serialize(&self.0.load(SeqCst), writer)
    }
}

/// Represents a Jio transaction
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub version: u16,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: SubnetworkId,
    pub gas: u64,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,

    /// Holds a commitment to the storage mass (KIP-0009)
    /// TODO: rename field and related methods to storage_mass
    #[serde(default)]
    mass: TransactionMass,

    // A field that is used to cache the transaction ID.
    // Always use the corresponding self.id() instead of accessing this field directly
    #[serde(with = "serde_bytes_fixed_ref")]
    id: TransactionId,
}

impl Transaction {
    pub fn new(
        version: u16,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        lock_time: u64,
        subnetwork_id: SubnetworkId,
        gas: u64,
        payload: Vec<u8>,
    ) -> Self {
        let mut tx = Self::new_non_finalized(version, inputs, outputs, lock_time, subnetwork_id, gas, payload);
        tx.finalize();
        tx
    }

    pub fn new_non_finalized(
        version: u16,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        lock_time: u64,
        subnetwork_id: SubnetworkId,
        gas: u64,
        payload: Vec<u8>,
    ) -> Self {
        Self { version, inputs, outputs, lock_time, subnetwork_id, gas, payload, mass: Default::default(), id: Default::default() }
    }

    pub fn validate(&self) -> Result<(), ConsensusError> {
        // Basic validation
        if self.version == 0 {
            return Err(ConsensusError::InvalidTransaction);
        }

        // Validate inputs
        if self.inputs.is_empty() {
            return Err(ConsensusError::InvalidTransaction);
        }

        // Validate outputs
        if self.outputs.is_empty() {
            return Err(ConsensusError::InvalidTransaction);
        }

        Ok(())
    }

    pub fn calculate_mass(&self) -> u64 {
        // Base mass for the transaction
        let mut mass = 100;

        // Add mass for each input
        mass += self.inputs.len() as u64 * 50;

        // Add mass for each output
        mass += self.outputs.len() as u64 * 30;

        // Add mass for payload
        mass += (self.payload.len() as u64 + 31) / 32 * 10;

        // Add gas cost to mass if it's a subnetwork transaction
        if self.subnetwork_id != SubnetworkId::default() {
            mass += self.gas;
        }

        mass
    }

    pub fn hash(&self) -> Hash {
        use crate::hashing::tx::calc_transaction_hash;
        calc_transaction_hash(self)
    }

    /// Determines whether or not a transaction is a coinbase transaction. A coinbase
    /// transaction is a special transaction created by miners that distributes fees and block subsidy
    /// to the previous blocks' miners, and specifies the script_pub_key that will be used to pay the current
    /// miner in future blocks.
    pub fn is_coinbase(&self) -> bool {
        self.subnetwork_id == subnets::SUBNETWORK_ID_COINBASE
    }

    /// Recompute and finalize the tx id based on updated tx fields
    pub fn finalize(&mut self) {
        self.id = hashing::tx::calc_transaction_hash(self);
    }

    /// Returns the transaction ID
    pub fn id(&self) -> TransactionId {
        self.id
    }

    /// Set the storage mass commitment field of this transaction. This field is expected to be activated on mainnet
    /// as part of the Crescendo hardfork. The field has no effect on tx ID so no need to finalize following this call.
    pub fn set_mass(&self, mass: u64) {
        self.mass.0.store(mass, SeqCst)
    }

    /// Read the storage mass commitment
    pub fn mass(&self) -> u64 {
        self.mass.0.load(SeqCst)
    }

    /// Set the storage mass commitment of the passed transaction
    pub fn with_mass(self, mass: u64) -> Self {
        self.set_mass(mass);
        self
    }
}

impl MemSizeEstimator for Transaction {
    fn estimate_mem_bytes(&self) -> usize {
        // Calculates mem bytes of the transaction (for cache tracking purposes)
        size_of::<Self>()
            + self.payload.len()
            + self
                .inputs
                .iter()
                .map(|i| i.signature_script.len() + size_of::<TransactionInput>())
                .chain(self.outputs.iter().map(|o| {
                    // size_of::<TransactionOutput>() already counts SCRIPT_VECTOR_SIZE bytes within, so we only add the delta
                    o.script_public_key.script().len().saturating_sub(SCRIPT_VECTOR_SIZE) + size_of::<TransactionOutput>()
                }))
                .sum::<usize>()
    }
}

/// Represents any kind of transaction which has populated UTXO entry data and can be verified/signed etc
pub trait VerifiableTransaction {
    fn tx(&self) -> &Transaction;

    /// Returns the `i`'th populated input
    fn populated_input(&self, index: usize) -> (&TransactionInput, &UtxoEntry);

    /// Returns an iterator over populated `(input, entry)` pairs
    fn populated_inputs(&self) -> PopulatedInputIterator<'_, Self>
    where
        Self: Sized,
    {
        PopulatedInputIterator::new(self)
    }

    fn inputs(&self) -> &[TransactionInput] {
        &self.tx().inputs
    }

    fn outputs(&self) -> &[TransactionOutput] {
        &self.tx().outputs
    }

    fn is_coinbase(&self) -> bool {
        self.tx().is_coinbase()
    }

    fn id(&self) -> TransactionId {
        self.tx().id()
    }

    fn utxo(&self, index: usize) -> Option<&UtxoEntry>;
}

/// A custom iterator written only so that `populated_inputs` has a known return type and can de defined on the trait level
pub struct PopulatedInputIterator<'a, T: VerifiableTransaction> {
    tx: &'a T,
    r: Range<usize>,
}

impl<T: VerifiableTransaction> Clone for PopulatedInputIterator<'_, T> {
    fn clone(&self) -> Self {
        Self { tx: self.tx, r: self.r.clone() }
    }
}

impl<'a, T: VerifiableTransaction> PopulatedInputIterator<'a, T> {
    pub fn new(tx: &'a T) -> Self {
        Self { tx, r: (0..tx.inputs().len()) }
    }
}

impl<'a, T: VerifiableTransaction> Iterator for PopulatedInputIterator<'a, T> {
    type Item = (&'a TransactionInput, &'a UtxoEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.r.next().map(|i| self.tx.populated_input(i))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.r.size_hint()
    }
}

impl<T: VerifiableTransaction> ExactSizeIterator for PopulatedInputIterator<'_, T> {}

/// Represents a read-only referenced transaction along with fully populated UTXO entry data
pub struct PopulatedTransaction<'a> {
    pub tx: &'a Transaction,
    pub entries: Vec<UtxoEntry>,
}

impl<'a> PopulatedTransaction<'a> {
    pub fn new(tx: &'a Transaction, entries: Vec<UtxoEntry>) -> Self {
        assert_eq!(tx.inputs.len(), entries.len());
        Self { tx, entries }
    }
}

impl VerifiableTransaction for PopulatedTransaction<'_> {
    fn tx(&self) -> &Transaction {
        self.tx
    }

    fn populated_input(&self, index: usize) -> (&TransactionInput, &UtxoEntry) {
        (&self.tx.inputs[index], &self.entries[index])
    }

    fn utxo(&self, index: usize) -> Option<&UtxoEntry> {
        self.entries.get(index)
    }
}

/// Represents a validated transaction with populated UTXO entry data and a calculated fee
pub struct ValidatedTransaction<'a> {
    pub tx: &'a Transaction,
    pub entries: Vec<UtxoEntry>,
    pub calculated_fee: u64,
}

impl<'a> ValidatedTransaction<'a> {
    pub fn new(populated_tx: PopulatedTransaction<'a>, calculated_fee: u64) -> Self {
        Self { tx: populated_tx.tx, entries: populated_tx.entries, calculated_fee }
    }

    pub fn new_coinbase(tx: &'a Transaction) -> Self {
        assert!(tx.is_coinbase());
        Self { tx, entries: Vec::new(), calculated_fee: 0 }
    }
}

impl VerifiableTransaction for ValidatedTransaction<'_> {
    fn tx(&self) -> &Transaction {
        self.tx
    }

    fn populated_input(&self, index: usize) -> (&TransactionInput, &UtxoEntry) {
        (&self.tx.inputs[index], &self.entries[index])
    }

    fn utxo(&self, index: usize) -> Option<&UtxoEntry> {
        self.entries.get(index)
    }
}

impl AsRef<Transaction> for Transaction {
    fn as_ref(&self) -> &Transaction {
        self
    }
}

/// Represents a generic mutable/readonly/pointer transaction type along
/// with partially filled UTXO entry data and optional fee and mass
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutableTransaction<T: AsRef<Transaction> = std::sync::Arc<Transaction>> {
    /// The inner transaction
    pub tx: T,
    /// Partially filled UTXO entry data
    pub entries: Vec<Option<UtxoEntry>>,
    /// Populated fee
    pub calculated_fee: Option<u64>,
    /// Populated non-contextual masses (does not include the storage mass)
    pub calculated_non_contextual_masses: Option<NonContextualMasses>,
}

impl<T: AsRef<Transaction>> MutableTransaction<T> {
    pub fn new(tx: T) -> Self {
        let num_inputs = tx.as_ref().inputs.len();
        Self { tx, entries: vec![None; num_inputs], calculated_fee: None, calculated_non_contextual_masses: None }
    }

    pub fn id(&self) -> TransactionId {
        self.tx.as_ref().id()
    }

    pub fn with_entries(tx: T, entries: Vec<UtxoEntry>) -> Self {
        assert_eq!(tx.as_ref().inputs.len(), entries.len());
        Self { tx, entries: entries.into_iter().map(Some).collect(), calculated_fee: None, calculated_non_contextual_masses: None }
    }

    /// Returns the tx wrapped as a [`VerifiableTransaction`]. Note that this function
    /// must be called only once all UTXO entries are populated, otherwise it panics.
    pub fn as_verifiable(&self) -> impl VerifiableTransaction + '_ {
        assert!(self.is_verifiable());
        MutableTransactionVerifiableWrapper { inner: self }
    }

    pub fn is_verifiable(&self) -> bool {
        assert_eq!(self.entries.len(), self.tx.as_ref().inputs.len());
        self.entries.iter().all(|e| e.is_some())
    }

    pub fn is_fully_populated(&self) -> bool {
        self.is_verifiable() && self.calculated_fee.is_some() && self.calculated_non_contextual_masses.is_some()
    }

    pub fn missing_outpoints(&self) -> impl Iterator<Item = TransactionOutpoint> + '_ {
        assert_eq!(self.entries.len(), self.tx.as_ref().inputs.len());
        self.entries.iter().enumerate().filter_map(|(i, entry)| {
            if entry.is_none() {
                Some(self.tx.as_ref().inputs[i].previous_outpoint)
            } else {
                None
            }
        })
    }

    pub fn clear_entries(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = None;
        }
    }

    /// Returns the calculated feerate. The feerate is calculated as the amount of fee this
    /// transactions pays per gram of the aggregated contextual mass (max over compute, transient
    /// and storage masses). The function returns a value when calculated fee and calculated masses
    /// exist, otherwise `None` is returned.
    pub fn calculated_feerate(&self) -> Option<f64> {
        self.calculated_non_contextual_masses
            .map(|non_contextual_masses| ContextualMasses::new(self.tx.as_ref().mass()).max(non_contextual_masses))
            .and_then(|max_mass| self.calculated_fee.map(|fee| fee as f64 / max_mass as f64))
    }

    /// A function for estimating the amount of memory bytes used by this transaction (dedicated to mempool usage).
    /// We need consistency between estimation calls so only this function should be used for this purpose since
    /// `estimate_mem_bytes` is sensitive to pointer wrappers such as Arc
    pub fn mempool_estimated_bytes(&self) -> usize {
        self.estimate_mem_bytes()
    }

    pub fn has_parent(&self, possible_parent: TransactionId) -> bool {
        self.tx.as_ref().inputs.iter().any(|x| x.previous_outpoint.transaction_id == possible_parent)
    }

    pub fn has_parent_in_set(&self, possible_parents: &HashSet<TransactionId>) -> bool {
        self.tx.as_ref().inputs.iter().any(|x| possible_parents.contains(&x.previous_outpoint.transaction_id))
    }
}

impl<T: AsRef<Transaction>> MemSizeEstimator for MutableTransaction<T> {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .entries
                .iter()
                .map(|op| {
                    // size_of::<Option<UtxoEntry>>() already counts SCRIPT_VECTOR_SIZE bytes within, so we only add the delta
                    size_of::<Option<UtxoEntry>>()
                        + op.as_ref().map_or(0, |e| e.script_public_key.script().len().saturating_sub(SCRIPT_VECTOR_SIZE))
                })
                .sum::<usize>()
            + self.tx.as_ref().estimate_mem_bytes()
    }
}

impl<T: AsRef<Transaction>> AsRef<Transaction> for MutableTransaction<T> {
    fn as_ref(&self) -> &Transaction {
        self.tx.as_ref()
    }
}

/// Private struct used to wrap a [`MutableTransaction`] as a [`VerifiableTransaction`]
struct MutableTransactionVerifiableWrapper<'a, T: AsRef<Transaction>> {
    inner: &'a MutableTransaction<T>,
}

impl<T: AsRef<Transaction>> VerifiableTransaction for MutableTransactionVerifiableWrapper<'_, T> {
    fn tx(&self) -> &Transaction {
        self.inner.tx.as_ref()
    }

    fn populated_input(&self, index: usize) -> (&TransactionInput, &UtxoEntry) {
        (
            &self.inner.tx.as_ref().inputs[index],
            self.inner.entries[index].as_ref().expect("expected to be called only following full UTXO population"),
        )
    }

    fn utxo(&self, index: usize) -> Option<&UtxoEntry> {
        self.inner.entries.get(index).and_then(Option::as_ref)
    }
}

/// Specialized impl for `T=Arc<Transaction>`
impl MutableTransaction {
    pub fn from_tx(tx: Transaction) -> Self {
        Self::new(std::sync::Arc::new(tx))
    }
}
