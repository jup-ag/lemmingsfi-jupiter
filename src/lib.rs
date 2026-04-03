pub mod quote;
pub mod state;

use solana_pubkey::Pubkey;

/// LemmingsFi program ID.
pub const PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("BQEJZUB4CzoT6UhRffoCkqCyqQNrCPCSGHcPEmsdbEsX");

pub use quote::{
    compute_swap_output, oracle_age_spread_penalty, QuoteError, QuoteInput, QuoteResult,
    SwapDirection,
};
pub use state::{deserialize_market, DeserializeError, GlobalConfigState, MarketState};
