#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]

//! Proof of Work in Iota

use crossbeam::Sender;
use failure::ensure;

use std::sync::{Arc, RwLock};

use iota_conversion::{Trinary, Trit};

use crossbeam::crossbeam_channel::unbounded;

type Result<T> = ::std::result::Result<T, failure::Error>;

/// State represents the various states that PearlDiver
/// will be in throughout its life
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PearlDiverState {
    /// Represents an instance of PearlDiver that hasn't been started yet.
    NotStarted,
    /// Represents an instance of PearlDiver that is currently running
    Running,
    /// Represents an instance of PearlDiver that has been cancelled
    Cancelled,
    /// Represents an instance of PearlDiver that has completed
    Completed,
}

const TRANSACTION_LENGTH: usize = 8019;
const CURL_HASH_LENGTH: usize = 243;
const CURL_STATE_LENGTH: usize = CURL_HASH_LENGTH * 3;
const HIGH_BITS: u64 =
    0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
const LOW_BITS: u64 =
    0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;

/// The PearlDiver struct allows you to start, stop, and check in on
/// PoW while its working
///```rust
/// use iota_client::extended::{PearlDiver, PowOptions};
/// use bee_crypto::ternary::{Curl, sponge};
/// use rand::{thread_rng, Rng};
///
/// const HASH_SIZE: usize = 243;
/// const MIN_WEIGHT_MAGNITUDE: usize = 9;
///
/// let mut rng = thread_rng();
/// let mut curl = Curl::default();
/// let trits: Vec<i8> = (0..8019).map(|_| rng.gen_range(-1, 2)).collect();
/// let mut pearl_diver = PearlDiver::default();
/// let result_trits = pearl_diver
///     .search(trits, PowOptions{min_weight_magnitude: 9, ..PowOptions::default()})
///     .unwrap();
/// let mut hash_trits = [0; HASH_SIZE];
/// curl.reset();
/// curl.absorb(&result_trits).unwrap();
/// curl.squeeze(&mut hash_trits).unwrap();
/// for j in (HASH_SIZE - MIN_WEIGHT_MAGNITUDE..HASH_SIZE - 1).rev() {
///     assert_eq!(hash_trits[j], 0);
/// }
///```
#[derive(Debug)]
pub struct PearlDiver {
    running: Arc<RwLock<PearlDiverState>>,
}

impl Default for PearlDiver {
    fn default() -> Self {
        PearlDiver {
            running: Arc::new(RwLock::new(PearlDiverState::NotStarted)),
        }
    }
}

/// Options of PoW configuration
#[derive(Debug)]
pub struct PowOptions {
    /// * `min_weight_magnitude` - Difficulty factor to use for Proof of Work
    pub min_weight_magnitude: usize,
    /// * `threads` - The number of threads to use
    pub threads: usize,
}

/// Provides reasonable defaults for PoW.
/// * `min_weight_magnitude` = 14
/// * `threads` = number of CPUs
impl Default for PowOptions {
    fn default() -> Self {
        PowOptions {
            min_weight_magnitude: 14,
            threads: num_cpus::get(),
        }
    }
}

impl PearlDiver {
    /// Creates a new PearlDiver instance
    pub fn new() -> PearlDiver {
        PearlDiver::default()
    }

    /// If you have multiple references to the same PearlDriver, this will allow
    /// you to cancel the proof of work. For this to be useful, you'll probably need
    /// to wrap the PearlDiver in an `Arc`
    ///```rust
    /// use iota_client::extended::PearlDiver;
    /// let mut pearl_diver = PearlDiver::new();
    /// // ... start running a pearl diver on another thread ...
    /// pearl_diver.cancel();
    ///```
    pub fn cancel(&mut self) {
        *self.running.write().unwrap() = PearlDiverState::Cancelled;
    }

    /// Returns the current status of PoW.
    pub fn status(&self) -> PearlDiverState {
        *self.running.read().unwrap()
    }

    /// Performs proof of work in place
    ///
    /// * `input` - Anything implementing the Trinary trait
    /// * `options` - PoW options
    pub fn search(&mut self, input: impl Trinary, options: PowOptions) -> Result<Vec<Trit>> {
        let transaction_trits = input.trits();
        let min_weight_magnitude = options.min_weight_magnitude;
        ensure!(
            transaction_trits.len() == TRANSACTION_LENGTH,
            "Transaction length [{}], expected [{}]",
            transaction_trits.len(),
            TRANSACTION_LENGTH
        );
        ensure!(
            min_weight_magnitude <= CURL_HASH_LENGTH,
            "Min Weight Magnitude must be less than {} but it is {}",
            min_weight_magnitude,
            CURL_HASH_LENGTH
        );
        *self.running.write().unwrap() = PearlDiverState::Running;

        let mut mid_state_low = [0; CURL_STATE_LENGTH];
        let mut mid_state_high = [0; CURL_STATE_LENGTH];
        initialize_mid_curl_states(&transaction_trits, &mut mid_state_low, &mut mid_state_high);

        let actual_thread_count = num_cpus::get();
        let threads = if options.threads == 0 {
            1
        } else if options.threads > actual_thread_count {
            actual_thread_count
        } else {
            options.threads
        };

        let (tx, rx) = unbounded();
        crossbeam::scope(|scope| {
            for _ in 0..threads {
                increment(
                    &mut mid_state_low,
                    &mut mid_state_high,
                    162 + CURL_HASH_LENGTH / 9,
                    162 + (CURL_HASH_LENGTH / 9) * 2,
                );
                let local_state_arc = Arc::clone(&self.running);
                let ref_tx_trits = &transaction_trits;
                let tx_clone = tx.clone();
                scope.spawn(move |_| {
                    get_runnable(
                        &local_state_arc,
                        &ref_tx_trits,
                        tx_clone,
                        min_weight_magnitude,
                        mid_state_low,
                        mid_state_high,
                    );
                });
            }
        })
        .unwrap();
        ensure!(
            *self.running.read().unwrap() == PearlDiverState::Completed,
            "Something went wrong."
        );
        let res = rx.recv().unwrap();
        Ok(res)
    }
}

fn get_runnable(
    state: &Arc<RwLock<PearlDiverState>>,
    transaction_trits: &[Trit],
    tx: Sender<Vec<Trit>>,
    min_weight_magnitude: usize,
    mut mid_state_copy_low: [u64; CURL_STATE_LENGTH],
    mut mid_state_copy_high: [u64; CURL_STATE_LENGTH],
) {
    let mut state_low = [0; CURL_STATE_LENGTH];
    let mut state_high = [0; CURL_STATE_LENGTH];

    let mut scratchpad_low = [0; CURL_STATE_LENGTH];
    let mut scratchpad_high = [0; CURL_STATE_LENGTH];

    let mask_start_index = CURL_HASH_LENGTH - min_weight_magnitude;
    let mut mask = 0;

    while mask == 0 && *state.read().unwrap() == PearlDiverState::Running {
        increment(
            &mut mid_state_copy_low,
            &mut mid_state_copy_high,
            162 + (CURL_HASH_LENGTH / 9) * 2,
            CURL_HASH_LENGTH,
        );
        copy(
            &mid_state_copy_low,
            &mid_state_copy_high,
            &mut state_low,
            &mut state_high,
        );
        transform(
            &mut state_low,
            &mut state_high,
            &mut scratchpad_low,
            &mut scratchpad_high,
        );

        mask = HIGH_BITS;
        for i in mask_start_index..CURL_HASH_LENGTH {
            mask &= !(state_low[i] ^ state_high[i]);
            if mask == 0 {
                break;
            }
        }
    }

    if mask != 0 && *state.read().unwrap() == PearlDiverState::Running {
        let mut out_mask = 1;
        while (out_mask & mask) == 0 {
            out_mask <<= 1;
        }
        let mut locked_transaction_trits = transaction_trits.to_vec();
        for i in 0..CURL_HASH_LENGTH {
            locked_transaction_trits[TRANSACTION_LENGTH - CURL_HASH_LENGTH + i] =
                if (mid_state_copy_low[i] & out_mask) == 0 {
                    1
                } else if (mid_state_copy_high[i] & out_mask) == 0 {
                    -1
                } else {
                    0
                };
        }
        tx.send(locked_transaction_trits).unwrap();
        *state.write().unwrap() = PearlDiverState::Completed;
    }
}

fn copy(src_low: &[u64], src_high: &[u64], dest_low: &mut [u64], dest_high: &mut [u64]) {
    dest_low[0..CURL_STATE_LENGTH].copy_from_slice(&src_low[0..CURL_STATE_LENGTH]);
    dest_high[0..CURL_STATE_LENGTH].copy_from_slice(&src_high[0..CURL_STATE_LENGTH]);
}

fn initialize_mid_curl_states(
    transaction_trits: &[Trit],
    mid_state_low: &mut [u64],
    mid_state_high: &mut [u64],
) {
    for i in CURL_HASH_LENGTH..CURL_STATE_LENGTH {
        mid_state_low[i] = HIGH_BITS;
        mid_state_high[i] = HIGH_BITS;
    }

    let mut offset = 0;
    let mut curl_scratchpad_low = [0; CURL_STATE_LENGTH];
    let mut curl_scratchpad_high = [0; CURL_STATE_LENGTH];
    for _ in (0..(TRANSACTION_LENGTH - CURL_HASH_LENGTH) / CURL_HASH_LENGTH).rev() {
        for j in 0..CURL_HASH_LENGTH {
            match transaction_trits[offset] {
                0 => {
                    mid_state_low[j] = HIGH_BITS;
                    mid_state_high[j] = HIGH_BITS;
                }
                1 => {
                    mid_state_low[j] = LOW_BITS;
                    mid_state_high[j] = HIGH_BITS;
                }
                _ => {
                    mid_state_low[j] = HIGH_BITS;
                    mid_state_high[j] = LOW_BITS;
                }
            }
            offset += 1;
        }
        transform(
            mid_state_low,
            mid_state_high,
            &mut curl_scratchpad_low,
            &mut curl_scratchpad_high,
        );
    }
    for i in 0..162 {
        match transaction_trits[offset] {
            0 => {
                mid_state_low[i] = HIGH_BITS;
                mid_state_high[i] = HIGH_BITS;
            }
            1 => {
                mid_state_low[i] = LOW_BITS;
                mid_state_high[i] = HIGH_BITS;
            }
            _ => {
                mid_state_low[i] = HIGH_BITS;
                mid_state_high[i] = LOW_BITS;
            }
        }
        offset += 1;
    }
    mid_state_low[162] =
        0b1101_1011_0110_1101_1011_0110_1101_1011_0110_1101_1011_0110_1101_1011_0110_1101;
    mid_state_high[162] =
        0b1011_0110_1101_1011_0110_1101_1011_0110_1101_1011_0110_1101_1011_0110_1101_1011;
    mid_state_low[162 + 1] =
        0b1111_0001_1111_1000_1111_1100_0111_1110_0011_1111_0001_1111_1000_1111_1100_0111;
    mid_state_high[162 + 1] =
        0b1000_1111_1100_0111_1110_0011_1111_0001_1111_1000_1111_1100_0111_1110_0011_1111;
    mid_state_low[162 + 2] =
        0b0111_1111_1111_1111_1110_0000_0000_1111_1111_1111_1111_1100_0000_0001_1111_1111;
    mid_state_high[162 + 2] =
        0b1111_1111_1100_0000_0001_1111_1111_1111_1111_1000_0000_0011_1111_1111_1111_1111;
    mid_state_low[162 + 3] =
        0b1111_1111_1100_0000_0000_0000_0000_0000_0000_0111_1111_1111_1111_1111_1111_1111;
    mid_state_high[162 + 3] =
        0b0000_0000_0011_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;
}

fn transform(
    state_low: &mut [u64],
    state_high: &mut [u64],
    scratchpad_low: &mut [u64],
    scratchpad_high: &mut [u64],
) {
    let mut scratch_index = 0;
    for _ in 0..81 {
        copy(state_low, state_high, scratchpad_low, scratchpad_high);
        for state_index in 0..CURL_STATE_LENGTH {
            let alpha = scratchpad_low[scratch_index];
            let beta = scratchpad_high[scratch_index];
            if scratch_index < 365 {
                scratch_index += 364;
            } else {
                scratch_index -= 365;
            }
            let gamma = scratchpad_high[scratch_index];
            let delta = (alpha | (!gamma)) & (scratchpad_low[scratch_index] ^ beta);

            state_low[state_index] = !delta;
            state_high[state_index] = (alpha ^ gamma) | delta;
        }
    }
}

fn increment(mid_low: &mut [u64], mid_high: &mut [u64], from_index: usize, to_index: usize) {
    let mut carry = 1;
    let mut low: u64;
    let mut hi: u64;
    let mut i = from_index;
    while i < to_index && carry != 0 {
        low = mid_low[i];
        hi = mid_high[i];
        mid_low[i] = hi ^ low;
        mid_high[i] = low;
        carry = hi & (!low);
        i += 1;
    }
}
