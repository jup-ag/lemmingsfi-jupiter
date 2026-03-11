/// Shared swap quoting engine for LemmingsFi.
/// Produces identical results to on-chain `compute_swap_output` in
/// `programs/lemmingsfi/src/instructions/swap.rs:183-235`.

pub const PRICE_SCALE: u64 = 1_000_000;
pub const BPS_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapDirection {
    /// User pays quote tokens, receives base tokens.
    BuyBase,
    /// User pays base tokens, receives quote tokens.
    SellBase,
}

/// Input parameters for a swap quote.
#[derive(Debug, Clone)]
pub struct QuoteInput {
    pub oracle_price: u64,
    pub bid_spread_bps: u16,
    pub ask_spread_bps: u16,
    pub fee_bps: u16,
}

/// Result of a swap quote.
#[derive(Debug, Clone)]
pub struct QuoteResult {
    /// Amount of output tokens.
    pub amount_out: u64,
    /// The effective price used (in PRICE_SCALE units).
    pub effective_price: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum QuoteError {
    #[error("Math overflow in swap computation")]
    MathOverflow,
    #[error("Zero output amount")]
    ZeroOutput,
}

/// Compute the output amount for a swap.
/// This must produce identical results to the on-chain `compute_swap_output` function.
pub fn compute_swap_output(
    input: &QuoteInput,
    direction: SwapDirection,
    amount_in: u64,
) -> Result<QuoteResult, QuoteError> {
    let price = input.oracle_price as u128;
    let bps = BPS_DENOMINATOR as u128;
    let scale = PRICE_SCALE as u128;
    let amount = amount_in as u128;

    match direction {
        SwapDirection::BuyBase => {
            let ask_spread = input.ask_spread_bps as u128;
            let fee = input.fee_bps as u128;

            // effective_ask = price * (bps + ask_spread) * (bps + fee) / (bps * bps)
            let effective_ask = price
                .checked_mul(
                    bps.checked_add(ask_spread)
                        .ok_or(QuoteError::MathOverflow)?,
                )
                .ok_or(QuoteError::MathOverflow)?
                .checked_mul(bps.checked_add(fee).ok_or(QuoteError::MathOverflow)?)
                .ok_or(QuoteError::MathOverflow)?
                .checked_div(bps.checked_mul(bps).ok_or(QuoteError::MathOverflow)?)
                .ok_or(QuoteError::MathOverflow)?;

            // base_out = quote_in * scale / effective_ask
            let base_out = amount
                .checked_mul(scale)
                .ok_or(QuoteError::MathOverflow)?
                .checked_div(effective_ask)
                .ok_or(QuoteError::MathOverflow)?;

            Ok(QuoteResult {
                amount_out: base_out as u64,
                effective_price: effective_ask as u64,
            })
        }
        SwapDirection::SellBase => {
            let bid_spread = input.bid_spread_bps as u128;
            let fee = input.fee_bps as u128;

            // effective_bid = price * (bps - bid_spread) * (bps - fee) / (bps * bps)
            let effective_bid = price
                .checked_mul(
                    bps.checked_sub(bid_spread)
                        .ok_or(QuoteError::MathOverflow)?,
                )
                .ok_or(QuoteError::MathOverflow)?
                .checked_mul(bps.checked_sub(fee).ok_or(QuoteError::MathOverflow)?)
                .ok_or(QuoteError::MathOverflow)?
                .checked_div(bps.checked_mul(bps).ok_or(QuoteError::MathOverflow)?)
                .ok_or(QuoteError::MathOverflow)?;

            // quote_out = base_in * effective_bid / scale
            let quote_out = amount
                .checked_mul(effective_bid)
                .ok_or(QuoteError::MathOverflow)?
                .checked_div(scale)
                .ok_or(QuoteError::MathOverflow)?;

            Ok(QuoteResult {
                amount_out: quote_out as u64,
                effective_price: effective_bid as u64,
            })
        }
    }
}

/// Convenience: create QuoteInput from a MarketState.
impl From<&crate::state::MarketState> for QuoteInput {
    fn from(market: &crate::state::MarketState) -> Self {
        Self {
            oracle_price: market.oracle_price,
            bid_spread_bps: market.bid_spread_bps,
            ask_spread_bps: market.ask_spread_bps,
            fee_bps: market.fee_bps,
        }
    }
}
