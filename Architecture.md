# stablechain Architecture

A signer-agnostic payroll CLI (`stablectl`) for native and ERC-20 transfers on the Tempo testnet, supporting three signing backends: a raw local key, a Privy app-controlled HSM wallet, and a Privy user-controlled wallet authenticated via browser login.

---

## Layer Diagram

```
┌──────────────────────────────────────────────────────┐
│                    stablectl (CLI)                    │
│         clap commands → App wiring → TxExecutor       │
└────────────────────────┬─────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────┐
│                   chain_access                      │
│                                                     │
│  ports/                                             │
│    ChainReader (trait)   ChainWriter (trait)        │
│                                                     │
│  adapters/                                          │
│    TempoAdapter  (implements ChainReader + Writer)  │
│    (tempo-alloy transport)                          │
│                                                     │
│  signer/                                            │
│    SignerBackend (trait)                            │
│    ├─ LocalKeySigner  (in-process ECDSA)            │
│    ├─ PrivySigner     (privy-rs SDK, app-controlled)│
│    └─ PrivyUserSigner (privy-rs SDK, user JWT)      │
│                                                     │
│  executor/                                          │
│    TxExecutor<R,W,S>  (send-flow orchestration)     │
└────────────────────────────────────────────────────┘
```

---

## Crate / Module Responsibilities

| Crate | Responsibility | Key types |
|-------|----------------|----------|
| `chain_access` | Chain interaction, signing, and send-flow orchestration | `ChainReader`, `ChainWriter`, `SignerBackend`, `TxExecutor`, `LocalKeySigner`, `PrivySigner`, `PrivyUserSigner`, `ChainId`, `ChainInfo` |
| `stablectl` | CLI parsing, wiring, output formatting | `Cli`, `App`, command handlers |

`TxExecutor<R, W, S>` is generic over all three port traits and lives in `chain_access::executor`. It is designed to be extracted to a separate crate without modification if needed.

---

## Workspace File Layout

```
stablechain/
├── Cargo.toml                        # workspace (members: chain_access, stablectl)
├── Architecture.md                   # this file
├── chain_access/
│   └── src/
│       ├── lib.rs
│       ├── error.rs                  # ChainAccessError (thiserror)
│       │
│       ├── domain/
│       │   ├── chain_id.rs           # ChainId enum (FromStr, Display, info())
│       │   ├── chain_info.rs         # ChainInfo trait — per-chain metadata
│       │   ├── chains/
│       │   │   └── tempo_testnet.rs  # TempoTestnet implementing ChainInfo
│       │   └── erc20.rs              # ERC-20 ABI calldata helpers (balanceOf, transfer)
│       ├── ports/
│       │   ├── chain_reader.rs       # ChainReader trait
│       │   └── chain_writer.rs       # ChainWriter trait
│       ├── adapters/
│       │   ├── tempo_provider.rs     # TempoProvider type alias + connect_tempo()
│       │   └── tempo_adapter.rs      # TempoAdapter (implements ChainReader + ChainWriter)
│       ├── signer/
│       │   ├── mod.rs                # SignerBackend trait
│       │   ├── local_key.rs          # LocalKeySigner
│       │   ├── privy.rs              # PrivySigner — app-controlled
│       │   └── privy_user.rs         # PrivyUserSigner — user JWT
│       └── executor/
│           └── mod.rs                # TxExecutor<R,W,S> — send-flow orchestration
│
└── stablectl/
    └── src/
        ├── main.rs
        ├── cli.rs                    # clap CLI definition
        ├── app.rs                    # App struct, executor wiring
        ├── commands/
        │   ├── chain.rs              # stablectl chain info
        │   ├── wallet.rs             # stablectl wallet balance native/erc20
        │   ├── tx.rs                 # stablectl tx transfer native/erc20
        │   ├── signer.rs             # stablectl signer privy create-wallet/wallet-info
        │   └── login.rs              # stablectl signer privy login/logout/whoami
        └── assets/
            └── login.html            # embedded Privy login page (PR 6)
```

---

## Core Trait Signatures

```rust
// chain_access/src/ports/chain_reader.rs
#[async_trait]
pub trait ChainReader: Send + Sync {
    fn chain_id(&self) -> ChainId;
    async fn native_balance(&self, address: Address) -> Result<U256, ChainAccessError>;
    async fn erc20_balance(&self, token: Address, owner: Address) -> Result<U256, ChainAccessError>;
    async fn nonce(&self, address: Address) -> Result<u64, ChainAccessError>;
    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<u64, ChainAccessError>;
    async fn gas_price(&self) -> Result<u128, ChainAccessError>;
    async fn block_number(&self) -> Result<u64, ChainAccessError>;
}

// chain_access/src/ports/chain_writer.rs
#[async_trait]
pub trait ChainWriter: Send + Sync {
    async fn send_raw_transaction(&self, rlp: Bytes) -> Result<TxHash, ChainAccessError>;
    async fn wait_for_receipt(&self, tx_hash: &TxHash) -> Result<TransactionReceipt, ChainAccessError>;
}

// chain_access/src/signer/mod.rs
#[async_trait]
pub trait SignerBackend: Send + Sync {
    async fn address(&self) -> Result<Address, ChainAccessError>;
    async fn sign_transaction(&self, tx: TransactionRequest) -> Result<Bytes, ChainAccessError>;
    fn signer_kind(&self) -> &'static str;
}
```

---

## Transaction Building

Transactions are built using `alloy::rpc::types::TransactionRequest`. ERC-20 calldata is encoded via helpers in `domain/erc20.rs`.

`TxExecutor` handles the full send-flow internally — callers just provide `to`, `amount`, and optionally `token`:

```rust
// stablectl command handler
let receipt = executor.send_native(to, amount).await?;
let receipt = executor.send_erc20(token, to, amount).await?;
```

Internally `TxExecutor::build_tx` constructs the `TransactionRequest`:

```rust
TransactionRequest::default()
    .with_chain_id(chain_id.info().chain_id())
    .with_from(sender)
    .with_to(to)
    .with_value(value)
    .with_nonce(nonce)
    .with_max_fee_per_gas(gas_price)
    .with_max_priority_fee_per_gas(gas_price / 10)
    .with_gas_limit(estimated_limit)
```

---

## Signer Backends

| Signer | `--signer` flag | Auth model | Key storage |
|--------|----------------|------------|-------------|
| `LocalKeySigner` | `local-key` | Raw ECDSA private key (`PRIVATE_KEY` env var) | In-process memory |
| `PrivySigner` | `privy` | App credentials (`PRIVY_APP_ID` + `PRIVY_APP_SECRET`). `AuthorizationContext::new()` (empty). | Privy HSM |
| `PrivyUserSigner` | `privy-user` | User JWT from browser login (`~/.stablectl/session.json`). `AuthorizationContext::new().push(JwtUser(...))`. | Privy HSM |

### Why `eth_signTransaction` (not `eth_sendTransaction`) for Privy

Privy's `eth_sendTransaction` broadcasts through Privy's own RPC infrastructure and requires a known CAIP-2 chain ID. Tempo uses a custom `tempo-alloy` transport that Privy's infrastructure doesn't support. Using `eth_signTransaction` (sign-only) embeds `chain_id: 42431` in the transaction fields, Privy signs offline, and we broadcast the RLP hex ourselves via `TempoProvider`.

---

## Chain Configuration

| Property | Value |
|----------|-------|
| Chain | Tempo testnet |
| Chain ID | `42431` |
| RPC URL | `https://rpc.moderato.tempo.xyz` |
| Explorer | `https://explore.tempo.xyz` |
| Tx URL | `https://explore.tempo.xyz/tx/{hash}` |
| Address URL | `https://explore.tempo.xyz/address/{address}` |

---

## Environment Variables

| Variable | Used by |
|----------|---------|
| `PRIVATE_KEY` | `--signer local-key` |
| `PRIVY_APP_ID` | All Privy commands |
| `PRIVY_APP_SECRET` | All Privy commands |
| `PRIVY_WALLET_ID` | `--signer privy` (app-controlled wallet) |
| `~/.stablectl/session.json` | `--signer privy-user` (written by `stablectl signer privy login`) |

---