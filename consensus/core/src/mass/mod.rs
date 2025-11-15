use crate::{
    config::params::Params,
    constants::TRANSIENT_BYTE_TO_MASS_FACTOR,
    subnets::SUBNETWORK_ID_SIZE,
    tx::{ScriptPublicKey, Transaction, TransactionInput, TransactionOutput, UtxoEntry, VerifiableTransaction},
};
use crate::HASH_SIZE;

// transaction_estimated_serialized_size is the estimated size of a transaction in some
// serialization. This has to be deterministic, but not necessarily accurate, since
// it's only used as the size component in the transaction and block mass limit
// calculation.
pub fn transaction_estimated_serialized_size(tx: &Transaction) -> u64 {
    let mut size: u64 = 0;
    size += 2; // Tx version (u16)
    size += 8; // Number of inputs (u64)
    let inputs_size: u64 = tx.inputs.iter().map(transaction_input_estimated_serialized_size).sum();
    size += inputs_size;

    size += 8; // number of outputs (u64)
    let outputs_size: u64 = tx.outputs.iter().map(transaction_output_estimated_serialized_size).sum();
    size += outputs_size;

    size += 8; // lock time (u64)
    size += SUBNETWORK_ID_SIZE as u64;
    size += 8; // gas (u64)
    size += HASH_SIZE as u64; // payload hash

    size += 8; // length of the payload (u64)
    size += tx.payload.len() as u64;
    size
}

fn transaction_input_estimated_serialized_size(input: &TransactionInput) -> u64 {
    let mut size = 0;
    size += outpoint_estimated_serialized_size();

    size += 8; // length of signature script (u64)
    size += input.signature_script.len() as u64;

    size += 8; // sequence (uint64)
    size
}

const fn outpoint_estimated_serialized_size() -> u64 {
    let mut size: u64 = 0;
    size += HASH_SIZE as u64; // Previous tx ID
    size += 4; // Index (u32)
    size
}

pub fn transaction_output_estimated_serialized_size(output: &TransactionOutput) -> u64 {
    let mut size: u64 = 0;
    size += 8; // value (u64)
    size += 2; // output.ScriptPublicKey.Version (u16)
    size += 8; // length of script public key (u64)
    size += output.script_public_key.script().len() as u64;
    size
}

/// Returns the UTXO storage "plurality" for this script public key.
/// i.e., how many 100-byte "storage units" it occupies.
/// The choice of 100 bytes per unit ensures that all standard SPKs have a plurality of 1.
pub fn utxo_plurality(spk: &ScriptPublicKey) -> u64 {
    /// A constant representing the number of bytes used by the fixed parts of a UTXO.
    const UTXO_CONST_STORAGE: usize =
        32  // outpoint::tx_id
        + 4 // outpoint::index
        + 8 // entry amount
        + 8 // entry DAA score
        + 1 // entry is coinbase
        + 2 // entry spk version
        + 8 // entry spk len
    ;

    // The base (63 bytes) plus the max standard public key length (33 bytes) fits into one 100-byte unit.
    // Hence, all standard SPKs end up with a plurality of 1.
    const UTXO_UNIT_SIZE: usize = 100;

    (UTXO_CONST_STORAGE + spk.script().len()).div_ceil(UTXO_UNIT_SIZE) as u64
}

pub trait UtxoPlurality {
    /// Returns the UTXO storage plurality for the script public key associated with this object.
    fn plurality(&self) -> u64;
}

impl UtxoPlurality for ScriptPublicKey {
    fn plurality(&self) -> u64 {
        utxo_plurality(self)
    }
}

impl UtxoPlurality for UtxoEntry {
    fn plurality(&self) -> u64 {
        utxo_plurality(&self.script_public_key)
    }
}

impl UtxoPlurality for TransactionOutput {
    fn plurality(&self) -> u64 {
        utxo_plurality(&self.script_public_key)
    }
}

/// An abstract UTXO storage cell.
///
/// # Plurality
///
/// Each `UtxoCell` now has a `plurality` field reflecting how many 100-byte "storage units"
/// this UTXO effectively occupies. This generalizes KIP-0009 to support UTXOs with
/// script public keys larger than the standard 33-byte limit. For a UTXO of byte-size
/// `entry.size`, we define:
///
/// ```ignore
/// p := ceil(entry.size / UTXO_UNIT)
/// ```
///
/// Conceptually, we treat a large UTXO as `p` sub-entries each holding `entry.amount / p`,
/// preserving the total locked amount but increasing the "count" proportionally to script size.
///
/// Refer to the KIP-0009 specification for more details.
#[derive(Clone, Copy)]
pub struct UtxoCell {
    /// The plurality (number of "storage units") for this UTXO
    pub plurality: u64,
    /// The amount of KLS (in sompis) locked in this UTXO
    pub amount: u64,
}

impl UtxoCell {
    pub fn new(plurality: u64, amount: u64) -> Self {
        Self { plurality, amount }
    }
}

impl From<&UtxoEntry> for UtxoCell {
    fn from(entry: &UtxoEntry) -> Self {
        Self::new(entry.plurality(), entry.amount)
    }
}

impl From<&TransactionOutput> for UtxoCell {
    fn from(output: &TransactionOutput) -> Self {
        Self::new(output.plurality(), output.value)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NonContextualMasses {
    /// Compute mass
    pub compute_mass: u64,

    /// Transient storage mass
    pub transient_mass: u64,
}

impl NonContextualMasses {
    pub fn new(compute_mass: u64, transient_mass: u64) -> Self {
        Self { compute_mass, transient_mass }
    }

    /// Returns the maximum over all non-contextual masses (currently compute and transient). This
    /// max value has no consensus meaning and should only be used for mempool-level simplification
    /// such as obtaining a one-dimensional mass value when composing blocks templates.  
    pub fn max(&self) -> u64 {
        self.compute_mass.max(self.transient_mass)
    }
}

impl std::fmt::Display for NonContextualMasses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "compute: {}, transient: {}", self.compute_mass, self.transient_mass)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ContextualMasses {
    /// Persistent storage mass
    pub storage_mass: u64,
}

impl ContextualMasses {
    pub fn new(storage_mass: u64) -> Self {
        Self { storage_mass }
    }

    /// Returns the maximum over *all masses* (currently compute, transient and storage). This max
    /// value has no consensus meaning and should only be used for mempool-level simplification such
    /// as obtaining a one-dimensional mass value when composing blocks templates.  
    pub fn max(&self, non_contextual_masses: NonContextualMasses) -> u64 {
        self.storage_mass.max(non_contextual_masses.max())
    }
}

impl std::fmt::Display for ContextualMasses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "storage: {}", self.storage_mass)
    }
}

impl std::cmp::PartialEq<u64> for ContextualMasses {
    fn eq(&self, other: &u64) -> bool {
        self.storage_mass.eq(other)
    }
}

pub type Mass = (NonContextualMasses, ContextualMasses);

pub trait MassOps {
    fn max(&self) -> u64;
}

impl MassOps for Mass {
    fn max(&self) -> u64 {
        self.1.max(self.0)
    }
}

// Note: consensus mass calculator operates on signed transactions.
// To calculate mass for unsigned transactions, please use
// `jio_wallet_core::tx::mass::MassCalculator`
#[derive(Clone)]
pub struct MassCalculator {
    mass_per_tx_byte: u64,
    mass_per_script_pub_key_byte: u64,
    mass_per_sig_op: u64,
    storage_mass_parameter: u64,
}

impl MassCalculator {
    pub fn new(mass_per_tx_byte: u64, mass_per_script_pub_key_byte: u64, mass_per_sig_op: u64, storage_mass_parameter: u64) -> Self {
        Self { mass_per_tx_byte, mass_per_script_pub_key_byte, mass_per_sig_op, storage_mass_parameter }
    }

    pub fn new_with_consensus_params(consensus_params: &Params) -> Self {
        Self {
            mass_per_tx_byte: consensus_params.mass_per_tx_byte,
            mass_per_script_pub_key_byte: consensus_params.mass_per_script_pub_key_byte,
            mass_per_sig_op: consensus_params.mass_per_sig_op,
            storage_mass_parameter: consensus_params.storage_mass_parameter,
        }
    }

    /// Calculates the non-contextual masses for this transaction (i.e., masses which can be calculated from
    /// the transaction alone). These include compute and transient storage masses of this transaction. This
    /// does not include the persistent storage mass calculation below which requires full UTXO context
    pub fn calc_non_contextual_masses(&self, tx: &Transaction) -> NonContextualMasses {
        if tx.is_coinbase() {
            return NonContextualMasses::new(0, 0);
        }

        let size = transaction_estimated_serialized_size(tx);
        let compute_mass_for_size = size * self.mass_per_tx_byte;
        let total_script_public_key_size: u64 = tx
            .outputs
            .iter()
            .map(|output| 2 /* script public key version (u16) */ + output.script_public_key.script().len() as u64)
            .sum();
        let total_script_public_key_mass = total_script_public_key_size * self.mass_per_script_pub_key_byte;

        let total_sigops: u64 = tx.inputs.iter().map(|input| input.sig_op_count as u64).sum();
        let total_sigops_mass = total_sigops * self.mass_per_sig_op;

        let compute_mass = compute_mass_for_size + total_script_public_key_mass + total_sigops_mass;
        let transient_mass = size * TRANSIENT_BYTE_TO_MASS_FACTOR;

        NonContextualMasses::new(compute_mass, transient_mass)
    }

    /// Calculates the contextual masses for this populated transaction.
    /// Assumptions which must be verified before this call:
    ///     1. All output values are non-zero
    ///     2. At least one input (unless coinbase)
    ///
    /// Otherwise this function should never fail.
    pub fn calc_contextual_masses(&self, tx: &impl VerifiableTransaction) -> Option<ContextualMasses> {
        calc_storage_mass(
            tx.is_coinbase(),
            tx.populated_inputs().map(|(_, entry)| entry.into()),
            tx.outputs().iter().map(|out| out.into()),
            self.storage_mass_parameter,
        )
        .map(ContextualMasses::new)
    }
}

/// Calculates the storage mass for the provided input and output values.
/// Calculates the storage mass (KIP-0009) for a given set of inputs and outputs.
///
/// This function has been generalized for UTXO entries that may exceed
/// the max standard 33-byte script public key size. Each `UtxoCell::plurality` indicates
/// how many 100-byte "storage units" that UTXO occupies.
///
/// # Formula Overview
///
/// The core formula is:
///
/// ```ignore
///     max(0, C · (|O| / H(O) - |I| / A(I)))
/// ```
///
/// where:
///
/// - `C` is the storage mass parameter (`storm_param`).
/// - `|O|` and `|I|` are the total pluralities of outputs and inputs, respectively.
/// - `H(O)` is the harmonic mean of the outputs' amounts, generalized to account for per-UTXO
///   `plurality`.
///
///   In standard KIP-0009, one has:
///
///   ```ignore
///   |O| / H(O) = Σ (1 / o)
///   ```
///
///   Here, each UTXO that occupies `p` storage units is treated as `p` sub-entries,
///   each holding `amount / p`. This effectively converts `1 / o` into `p^2 / amount`.
///   Consequently, the code accumulates:
///
///   ```ignore
///   Σ [C · p(o)^2 / amount(o)]
///   ```
///
/// - `A(I)` is the arithmetic mean of the inputs' amounts, similarly scaled by `|I|`,
///   while the sum of amounts remains unchanged.
///
/// Under the “relaxed formula” conditions (`|O| = 1`, `|I| = 1`, or `|O| = |I| = 2`),
/// we compute the harmonic mean for inputs as well; otherwise, we use the arithmetic
/// approach for inputs.
///
/// Refer to KIP-0009 for more details.
///
/// Assumptions which must be verified before this call:
///   1. All input/output values are non-zero
///   2. At least one input (unless coinbase)
///
/// If these assumptions hold, this function should never fail. A `None` return
/// indicates that the mass is incomputable and can be considered too high.
pub fn calc_storage_mass(
    is_coinbase: bool,
    inputs: impl ExactSizeIterator<Item = UtxoCell> + Clone,
    mut outputs: impl Iterator<Item = UtxoCell>,
    storm_param: u64,
) -> Option<u64> {
    if is_coinbase {
        return Some(0);
    }

    /*
        In KIP-0009 terms, the canonical formula is:
            max(0, C * (|O|/H(O) - |I|/A(I))).

        We first calculate the harmonic portion for outputs in a single pass,
        accumulating:
            1) outs_plurality = Σ p(o)
            2) harmonic_outs  = Σ [C * p(o)^2 / amount(o)]
    */
    let (outs_plurality, harmonic_outs) = outputs.try_fold(
        (0u64, 0u64), // (accumulated plurality, accumulated harmonic)
        |(acc_plurality, acc_harm), UtxoCell { plurality, amount }| {
            Some((
                acc_plurality + plurality, // represents in-memory bytes, cannot overflow
                acc_harm.checked_add(storm_param.checked_mul(plurality)?.checked_mul(plurality)? / amount)?,
            ))
        },
    )?;

    /*
        KIP-0009 defines a relaxed formula for the cases:
            |O| = 1  or  |O| <= |I| <= 2

        The relaxed formula is:
            max(0, C · (|O| / H(O) - |I| / H(I)))

        If |I| = 1, the harmonic and arithmetic approaches coincide, so the conditions can be expressed as:
            |O| = 1 or |I| = 1 or |O| = |I| = 2
    */
    let relaxed_formula_path = {
        if outs_plurality == 1 {
            true // |O| = 1
        } else if inputs.len() > 2 {
            false // since element plurality always >= 1 => ins_plurality > 2 => skip harmonic path
        } else {
            // For <= 2 inputs, we can afford to clone and sum the pluralities
            let ins_plurality = inputs.clone().map(|cell| cell.plurality).sum::<u64>();
            ins_plurality == 1 || (outs_plurality == 2 && ins_plurality == 2)
        }
    };

    if relaxed_formula_path {
        // Each input i contributes C · p(i)^2 / amount(i)
        let harmonic_ins = inputs
            .map(|UtxoCell { plurality, amount }| storm_param * plurality * plurality / amount) // we assume no overflow (see verify_utxo_plurality_limits)
            .fold(0u64, |total, current| total.saturating_add(current));
        // max(0, harmonic_outs - harmonic_ins)
        return Some(harmonic_outs.saturating_sub(harmonic_ins));
    }

    // Otherwise, we calculate the arithmetic portion for inputs:
    // (ins_plurality, sum_ins) =>  (Σ plurality, Σ amounts)
    let (ins_plurality, sum_ins) =
        inputs.fold((0u64, 0u64), |(acc_plur, acc_amt), UtxoCell { plurality, amount }| (acc_plur + plurality, acc_amt + amount));

    // mean_ins = (Σ amounts) / (Σ plurality)
    let mean_ins = sum_ins / ins_plurality;

    // arithmetic_ins:  C · (|I| / A(I)) = |I| · (C / mean_ins)
    let arithmetic_ins = ins_plurality.saturating_mul(storm_param / mean_ins);

    // max(0, harmonic_outs - arithmetic_ins)
    Some(harmonic_outs.saturating_sub(arithmetic_ins))
}
