# stablechain Justfile — run `just` to list all tasks.
set dotenv-load  # auto-loads .env if present

RPC       := "https://rpc.moderato.tempo.xyz"
STABLECTL := "./target/debug/stablectl"
CHAIN     := "tempo-testnet"

# ── demo wallet addresses ────────────────────────────────────────────────────

WALLET_A  := "0x1739281f86FA1915bCd52FC06A4A1CcBCBeb2Ac3"
WALLET_B  := "0x946f856d4e7D5f35A6E74e74146aFa435b6483d2"

# ── pre-deployed stablecoins (Tempo testnet) ─────────────────────────────────

PATH_USD  := "0x20c0000000000000000000000000000000000000"
ALPHA_USD := "0x20c0000000000000000000000000000000000001"
BETA_USD  := "0x20c0000000000000000000000000000000000002"
THETA_USD := "0x20c0000000000000000000000000000000000003"

# ── default ──────────────────────────────────────────────────────────────────

[private]
default:
    @just --list

# ── build & test ─────────────────────────────────────────────────────────────

# Build stablectl binary
build:
    cargo build -p stablectl

# Build in release mode
build-release:
    cargo build -p stablectl --release

# Run all tests
test:
    cargo test

# Fast type-check without linking
check:
    cargo check

# ── demo ─────────────────────────────────────────────────────────────────────

# Run the interactive demo
demo: build
    bash demo.sh

# ── stablectl queries ────────────────────────────────────────────────────────

# Print chain info and current block
chain-info chain=CHAIN: build
    {{STABLECTL}} --chain {{chain}} chain info

# Native token balance  —  just balance-native  OR  just balance-native 0xADDR
balance-native addr=WALLET_A chain=CHAIN: build
    {{STABLECTL}} --chain {{chain}} wallet balance native --address {{addr}}

# ERC-20 balance  —  just balance-erc20  OR  just balance-erc20 0xTOKEN 0xADDR 18
balance-erc20 token=PATH_USD addr=WALLET_A decimals="6" chain=CHAIN: build
    {{STABLECTL}} --chain {{chain}} wallet balance erc20 --token {{token}} --address {{addr}} --decimals {{decimals}}

# ── faucet ───────────────────────────────────────────────────────────────────

# Fund an address with 1M of each stablecoin  —  just faucet  OR  just faucet 0xADDR
faucet addr=WALLET_A:
    curl -s -X POST {{RPC}} \
      -H "Content-Type: application/json" \
      -d '{"jsonrpc":"2.0","method":"tempo_fundAddress","params":["{{addr}}"],"id":1}' \
      | python3 -m json.tool

# Fund both demo wallets
faucet-all:
    just faucet {{WALLET_A}}
    just faucet {{WALLET_B}}

# ── cast shortcuts ───────────────────────────────────────────────────────────

# Raw native balance via cast
cast-balance addr=WALLET_A:
    cast balance {{addr}} --rpc-url {{RPC}}

# Raw ERC-20 balance via cast
cast-erc20 token=PATH_USD addr=WALLET_A:
    cast call {{token}} "balanceOf(address)(uint256)" {{addr}} --rpc-url {{RPC}}

# Current block number
cast-block:
    cast block-number --rpc-url {{RPC}}

# Decode a transaction by hash
cast-tx hash:
    cast tx {{hash}} --rpc-url {{RPC}}

# ── keys (loaded from .env — see .env.example) ───────────────────────────────

# Show demo wallet addresses
wallets:
    @echo "Wallet A: {{WALLET_A}}"
    @echo "Wallet B: {{WALLET_B}}"
