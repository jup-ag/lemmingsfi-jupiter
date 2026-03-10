# LemmingsFi Market Account Layout

## Program ID

```
BQEJZUB4CzoT6UhRffoCkqCyqQNrCPCSGHcPEmsdbEsX
```

## PDA Seeds

| Account | Seeds | Description |
|---------|-------|-------------|
| GlobalConfig | `["config"]` | Singleton config |
| Market | `["market", base_mint, quote_mint]` | Per-pair market |
| VaultBase | `["vault_base", market]` | Base token vault (PDA-owned) |
| VaultQuote | `["vault_quote", market]` | Quote token vault (PDA-owned) |

## Market Account (8-byte Anchor discriminator + 234 bytes data = 242 bytes total)

| Offset | Size | Type | Field | Description |
|--------|------|------|-------|-------------|
| 0 | 8 | `[u8; 8]` | discriminator | Anchor discriminator (`sha256("account:Market")[..8]`) |
| 8 | 32 | Pubkey | base_mint | Base token mint (e.g., SOL) |
| 40 | 32 | Pubkey | quote_mint | Quote token mint (e.g., USDC) |
| 72 | 32 | Pubkey | vault_base | PDA-owned base token account |
| 104 | 32 | Pubkey | vault_quote | PDA-owned quote token account |
| 136 | 32 | Pubkey | authority | Market authority (oracle updater) |
| 168 | 8 | u64 | oracle_price | Oracle price (6-decimal fixed-point, e.g., `150_123_456` = $150.123456) |
| 176 | 8 | u64 | oracle_conf | Confidence interval (same scale as price) |
| 184 | 8 | i64 | oracle_timestamp | Unix timestamp of last oracle update |
| 192 | 8 | u64 | oracle_slot | Solana slot of last oracle update |
| 200 | 2 | u16 | bid_spread_bps | Bid spread in basis points (100 = 1%) |
| 202 | 2 | u16 | ask_spread_bps | Ask spread in basis points |
| 204 | 2 | u16 | fee_bps | Taker fee in basis points (30 = 0.3%) |
| 206 | 8 | u64 | min_order_size | Min order in base token smallest units (0 = no limit) |
| 214 | 8 | u64 | max_order_size | Max order in base token smallest units (0 = no limit) |
| 222 | 8 | u64 | concentration | Liquidity concentration parameter |
| 230 | 8 | u64 | max_staleness_slots | Oracle staleness limit in slots |
| 238 | 2 | u16 | max_price_deviation_bps | Max per-update price change in bps |
| 240 | 1 | bool | paused | Market-level pause flag |
| 241 | 1 | u8 | bump | PDA bump seed |

**Total: 242 bytes** (8 discriminator + 234 data)

## GlobalConfig Account (8-byte discriminator + 68 bytes data = 76 bytes total)

| Offset | Size | Type | Field | Description |
|--------|------|------|-------|-------------|
| 0 | 8 | `[u8; 8]` | discriminator | Anchor discriminator (`sha256("account:GlobalConfig")[..8]`) |
| 8 | 32 | Pubkey | authority | Protocol authority |
| 40 | 32 | Pubkey | fee_recipient | Fee collection address |
| 72 | 2 | u16 | default_fee_bps | Default taker fee |
| 74 | 1 | bool | paused | Global pause flag |
| 75 | 1 | u8 | bump | PDA bump seed |

## Constants

| Name | Value | Description |
|------|-------|-------------|
| PRICE_SCALE | 1,000,000 | 6-decimal fixed-point denominator |
| BPS_DENOMINATOR | 10,000 | Basis points denominator |

## Byte Encoding

All multi-byte integers are **little-endian**. Pubkeys are 32 raw bytes. Booleans are 1 byte (0 = false, 1 = true). Uses Borsh serialization.
