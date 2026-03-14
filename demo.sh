#!/usr/bin/env bash
# demo.sh — stablechain end-to-end demo
# Grows with each PR; sections are added as features land.
set -euo pipefail

# ── helpers ─────────────────────────────────────────────────────────────────

BOLD='\033[1m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
DIM='\033[2m'
RESET='\033[0m'

section() { echo -e "\n${BOLD}${CYAN}▶ $1${RESET}"; }
cmd()     { echo -e "  ${GREEN}\$ $*${RESET}"; "$@"; }
note()    { echo -e "  ${DIM}# $*${RESET}"; }
pause()   { echo -e "\n${DIM}  Press Enter to continue...${RESET}"; read -r; }

STABLECTL="./target/debug/stablectl"
CHAIN="tempo-testnet"

# ── build ────────────────────────────────────────────────────────────────────

echo -e "${BOLD}Building stablectl...${RESET}"
cargo build -q -p stablectl
echo

# ── PR 2 — Read-only queries ─────────────────────────────────────────────────

section "Chain info"
cmd $STABLECTL --chain "$CHAIN" chain info

pause

section "Native balance"
WALLET_A="0x1739281f86FA1915bCd52FC06A4A1CcBCBeb2Ac3"
cmd $STABLECTL --chain "$CHAIN" wallet balance native --address "$WALLET_A"

pause

section "ERC-20 balance (pathUSD — 6 decimals)"
PATH_USD="0x20c0000000000000000000000000000000000000"
cmd $STABLECTL --chain "$CHAIN" wallet balance erc20 \
    --token    "$PATH_USD" \
    --address  "$WALLET_A" \
    --decimals 6

# ── PR 3 — Local-key send (coming soon) ──────────────────────────────────────

# pause
# section "Native transfer"
# cmd $STABLECTL --chain "$CHAIN" wallet send native \
#     --signer local-key --key-env DEMO_WALLET_A_KEY \
#     --to "$WALLET_B"  --amount 0.001

# pause
# section "ERC-20 transfer"
# cmd $STABLECTL --chain "$CHAIN" wallet send erc20 \
#     --signer local-key --key-env DEMO_WALLET_A_KEY \
#     --token "$PATH_USD" --to "$WALLET_B" --amount 1.0

# ── PR 4+ — Privy signer (coming soon) ───────────────────────────────────────

# pause
# section "App-controlled wallet — create"
# cmd $STABLECTL --chain "$CHAIN" wallet create --signer privy-app

# pause
# section "App-controlled wallet — send"
# cmd $STABLECTL --chain "$CHAIN" wallet send native \
#     --signer privy-app \
#     --to "$WALLET_B" --amount 0.001

echo -e "\n${BOLD}Done.${RESET}\n"
