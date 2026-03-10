# DFlow Integration Guide for LemmingsFi

## Overview

LemmingsFi is an oracle-based AMM on Solana. It uses off-chain oracle updates to set bid/ask prices with configurable spreads and fees. DFlow can read market state directly from on-chain accounts via LaserStream/geyser and route JIT orders through the swap instruction.

## Program ID

```
BQEJZUB4CzoT6UhRffoCkqCyqQNrCPCSGHcPEmsdbEsX
```

## Architecture

```
Oracle Updater (off-chain) ---> Market Account (on-chain)
                                       |
DFlow LaserStream ---reads--->  Market + Vaults
                                       |
DFlow JIT Router  ---calls--->  Swap Instruction
```

- The oracle updater submits `update_oracle` transactions to keep prices fresh
- DFlow reads `Market` state + vault balances to compute quotes
- DFlow calls the `swap` instruction directly for JIT execution
- `min_amount_out` in the swap instruction provides slippage protection for rerouting safety

## Account Layout

See [market-layout.md](market-layout.md) for complete byte-offset tables.

## Swap Math

### BuyBase (user pays quote, receives base)

```
effective_ask = oracle_price * (10000 + ask_spread_bps) * (10000 + fee_bps) / (10000 * 10000)
base_out = quote_in * PRICE_SCALE / effective_ask
```

### SellBase (user pays base, receives quote)

```
effective_bid = oracle_price * (10000 - bid_spread_bps) * (10000 - fee_bps) / (10000 * 10000)
quote_out = base_in * effective_bid / PRICE_SCALE
```

Where:
- `PRICE_SCALE = 1,000,000` (6-decimal fixed-point)
- All intermediate math uses `u128` to prevent overflow
- Integer division truncates (floor)

### Order Size Checks

After computing the output amount:
- `base_amount` = `base_out` (BuyBase) or `base_in` (SellBase)
- If `min_order_size > 0`: reject if `base_amount < min_order_size`
- If `max_order_size > 0`: reject if `base_amount > max_order_size`

### Liquidity Checks

- BuyBase: `vault_base.amount >= base_out`
- SellBase: `vault_quote.amount >= quote_out`

### Staleness Check

Swaps are rejected if `current_slot - market.oracle_slot > market.max_staleness_slots`.

## Swap Instruction

### Discriminator

```
sha256("global:swap")[..8]
```

### Instruction Data (25 bytes)

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 8 | `[u8; 8]` | discriminator |
| 8 | 1 | u8 | direction (0 = BuyBase, 1 = SellBase) |
| 9 | 8 | u64 | amount_in |
| 17 | 8 | u64 | min_amount_out |

### Account Ordering (8 accounts)

| Index | Account | Writable | Signer | Description |
|-------|---------|----------|--------|-------------|
| 0 | user | No | Yes | User wallet |
| 1 | global_config | No | No | GlobalConfig PDA |
| 2 | market | Yes | No | Market PDA |
| 3 | vault_base | Yes | No | Base token vault |
| 4 | vault_quote | Yes | No | Quote token vault |
| 5 | user_base | Yes | No | User's base token account |
| 6 | user_quote | Yes | No | User's quote token account |
| 7 | token_program | No | No | SPL Token program (`TokenkegQEqKXcWGS9sM...`) |

**For BuyBase**: user_base = destination (receives base), user_quote = source (pays quote)
**For SellBase**: user_base = source (pays base), user_quote = destination (receives quote)

### SwapDirection Encoding

Borsh enum encoding:
- `BuyBase` = `0x00`
- `SellBase` = `0x01`

## Error Codes

| Code | Name | Hex | Description |
|------|------|-----|-------------|
| 6000 | Unauthorized | 0x1770 | Caller is not authority |
| 6001 | MarketPaused | 0x1771 | Market-level pause |
| 6002 | GlobalPaused | 0x1772 | Global kill switch |
| 6003 | StaleOracle | 0x1773 | Oracle too old |
| 6004 | PriceDeviationTooLarge | 0x1774 | Price jump too large |
| 6005 | InvalidOraclePrice | 0x1775 | Zero price |
| 6006 | SlippageExceeded | 0x1776 | Output below min_amount_out |
| 6007 | OrderTooSmall | 0x1777 | Below min_order_size |
| 6008 | OrderTooLarge | 0x1778 | Above max_order_size |
| 6009 | InsufficientLiquidity | 0x1779 | Vault can't cover output |
| 6010 | MathOverflow | 0x177A | Arithmetic overflow |
| 6011 | InvalidFee | 0x177B | Fee > 10000 bps |
| 6012 | InvalidSpread | 0x177C | Spread > 10000 bps |
| 6013 | ZeroDeposit | 0x177D | Zero deposit amount |
| 6014 | ZeroWithdraw | 0x177E | Zero withdraw amount |
| 6015 | InvalidSwapDirection | 0x177F | Bad direction enum value |

## Reading Market State for Quoting

1. Fetch the `Market` account data (242 bytes)
2. Skip 8-byte Anchor discriminator
3. Deserialize fields using Borsh (little-endian)
4. Read `oracle_price`, `bid_spread_bps`, `ask_spread_bps`, `fee_bps`
5. Fetch vault token accounts to check available liquidity:
   - `vault_base` balance = `u64` at byte offset 64 of SPL Token account
   - `vault_quote` balance = same

## Discovering Markets

Markets are created as PDAs with seeds `["market", base_mint, quote_mint]`. To discover all markets:
1. Use `getProgramAccounts` with `memcmp` on the 8-byte discriminator
2. Or use the Anchor IDL (`target/idl/lemmingsfi.json`) with standard Anchor account discovery

## Anchor IDL

The Anchor IDL is generated by `anchor build` and available at `target/idl/lemmingsfi.json`. It provides full type-safe account and instruction definitions.

## Contact

For integration support: contact the LemmingsFi team with:
1. This document
2. The Anchor IDL file
3. List of live Market PDAs and their trading pairs
