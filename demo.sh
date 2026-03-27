#!/usr/bin/env bash
# demo.sh — stablechain end-to-end demo
# Full walkthrough of stablectl capabilities for the Tempo testnet.
set -euo pipefail

# ── helpers ─────────────────────────────────────────────────────────────────

BOLD='\033[1m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
DIM='\033[2m'
RESET='\033[0m'

section() { printf "\n${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n"; printf "${BOLD}${CYAN}▶ %s${RESET}\n\n" "$1"; }
cmd()     { printf "  ${GREEN}\$ %s${RESET}\n" "$*"; "$@"; }
note()    { printf "  ${DIM}%s${RESET}\n" "$*"; }
warn()    { printf "  ${YELLOW}⚠ %s${RESET}\n" "$*"; }
pause()   { printf "\n${DIM}  ─── Press Enter to continue ───${RESET}\n"; read -r; }
banner()  {
  printf "${BOLD}${CYAN}\n"
  echo "  ┌──────────────────────────────────────────────────┐"
  echo "  │                                                  │"
  echo "  │       stablechain CLI  —  Live Demo              │"
  echo "  │       Signer-agnostic transfers on Tempo         │"
  echo "  │                                                  │"
  echo "  └──────────────────────────────────────────────────┘"
  printf "${RESET}\n"
}

STABLECTL="./target/debug/stablectl"
CHAIN="tempo-testnet"

WALLET_A="0x1739281f86FA1915bCd52FC06A4A1CcBCBeb2Ac3"
WALLET_B="0x4FE210d1A43D3D49D636cB33E142223a493885E1"
PATH_USD="0x20c0000000000000000000000000000000000000"
ALPHA_USD="0x20c0000000000000000000000000000000000001"

# ── pre-flight checks ───────────────────────────────────────────────────────

banner

note "This demo walks through every stablectl feature:"
note "  1. Chain info            — read on-chain metadata"
note "  2. Token registry        — list supported TIP-20 stablecoins"
note "  3. Balance queries       — native + ERC-20 balances"
note "  4. Local-key send        — sign & broadcast with a local private key"
note "  5. Privy browser login   — authenticate via Privy SDK in the browser"
note "  6. Privy signer session  — check session status (whoami)"
note "  7. Privy wallet create   — create a server-managed wallet via Privy"
note "  8. Fund Privy wallet     — send tokens from local-key wallet to the new wallet"
note "  9. Privy wallet send     — sign with Privy server signer"
note ""
note "Prerequisites:"
note "  - DEMO_WALLET_A_KEY        (hex private key for local-key demos)"
note "  - PRIVY_TEST_APP_ID        (Privy app ID)"
note "  - PRIVY_TEST_APP_SECRET    (Privy app secret)"
echo ""

# ── build ────────────────────────────────────────────────────────────────────

printf "${BOLD}Building stablectl...${RESET}\n"
cargo build -q -p stablectl
printf "${GREEN}Build successful.${RESET}\n"

pause

###############################################################################
#  PART 1 — READ-ONLY CHAIN QUERIES
###############################################################################

section "1/9  Chain info"
note "Connect to the Tempo testnet and display chain metadata + latest block."
echo ""
cmd $STABLECTL --chain "$CHAIN" chain info

pause

section "2/9  Supported TIP-20 tokens"
note "Tempo uses TIP-20 stablecoins instead of a native gas token."
note "stablectl ships with a built-in registry of supported tokens."
echo ""
cmd $STABLECTL wallet tokens

pause

section "3/9  Balance queries"
note "Query native (TEMPO) balance and ERC-20 (pathUSD) balance for a wallet."
echo ""

note "── Native balance ──"
cmd $STABLECTL --chain "$CHAIN" wallet balance native --address "$WALLET_A"
echo ""

note "── pathUSD balance (6 decimals) ──"
cmd $STABLECTL --chain "$CHAIN" wallet balance erc20 \
    --token    "$PATH_USD" \
    --address  "$WALLET_A" \
    --decimals 6

pause

###############################################################################
#  PART 2 — LOCAL-KEY SIGNER (send with a private key from env)
###############################################################################

section "4/9  ERC-20 transfer with local-key signer"
note "Sign and broadcast a pathUSD transfer using a local private key."
note "The key is read from the DEMO_WALLET_A_KEY environment variable."
echo ""

if [ -z "${DEMO_WALLET_A_KEY:-}" ]; then
  warn "DEMO_WALLET_A_KEY is not set — skipping local-key send."
  note "To enable: export DEMO_WALLET_A_KEY=0x<64-hex-char-private-key>"
elif [ ${#DEMO_WALLET_A_KEY} -lt 64 ]; then
  warn "DEMO_WALLET_A_KEY looks too short (${#DEMO_WALLET_A_KEY} chars). A private key should be 66 chars (0x + 64 hex)."
  note "You may have set an address instead of a private key. Skipping."
else
  cmd $STABLECTL --chain "$CHAIN" wallet send erc20 \
      --signer local-key --key-env DEMO_WALLET_A_KEY \
      --token "$PATH_USD" \
      --to "$WALLET_B" \
      --amount 0.01 \
      --decimals 6
fi

pause

###############################################################################
#  PART 3 — PRIVY SIGNER (browser login, session, wallet create, send)
###############################################################################

section "5/9  Privy browser login"
note "Opens a local browser page powered by the Privy React SDK."
note "Login via email or wallet — the JWT is captured automatically."
note "No manual token paste needed!"
echo ""

if [ -z "${PRIVY_TEST_APP_ID:-}" ] || [ -z "${PRIVY_TEST_APP_SECRET:-}" ]; then
  warn "PRIVY_TEST_APP_ID / PRIVY_TEST_APP_SECRET not set — skipping Privy steps."
  note "To enable: export PRIVY_TEST_APP_ID=... PRIVY_TEST_APP_SECRET=..."
  printf "\n${BOLD}Skipping steps 5-9 (Privy). Done.${RESET}\n"
  exit 0
fi

cmd $STABLECTL signer login-browser --port 8787
echo ""
note "Session saved. The JWT is stored locally for subsequent commands."

pause

section "6/9  Signer session status (whoami)"
note "Check who is currently logged in and where the session file lives."
echo ""
cmd $STABLECTL signer whoami

pause

section "7/9  Create a Privy server-managed wallet"
note "Ask Privy to provision a new Ethereum wallet. The private key"
note "lives on Privy's infrastructure — stablectl never sees it."
echo ""

# Run wallet create and capture output so we can auto-extract the ID and address.
CREATE_OUTPUT=$($STABLECTL wallet create 2>&1)
echo "$CREATE_OUTPUT"

# Parse Wallet ID and Address from the table output.
PRIVY_WALLET_ID=$(echo "$CREATE_OUTPUT"  | grep -i 'Wallet ID'      | awk '{print $NF}')
PRIVY_WALLET_ADDR=$(echo "$CREATE_OUTPUT" | grep -i 'Wallet Address' | awk '{print $NF}')

if [ -z "$PRIVY_WALLET_ID" ] || [ -z "$PRIVY_WALLET_ADDR" ]; then
  warn "Could not parse wallet ID or address from output — skipping steps 8-9."
else
  echo ""
  note "Auto-captured:  ID = ${PRIVY_WALLET_ID}"
  note "                Addr = ${PRIVY_WALLET_ADDR}"

  pause

  section "8/9  Fund the new Privy wallet"
  note "Transfer pathUSD from the local-key wallet to the newly created"
  note "Privy wallet so it has tokens to send in the next step."
  echo ""

  cmd $STABLECTL --chain "$CHAIN" wallet send erc20 \
      --signer local-key --key-env DEMO_WALLET_A_KEY \
      --token "$PATH_USD" \
      --to "$PRIVY_WALLET_ADDR" \
      --amount 2.0 \
      --decimals 6

  pause

  section "9/9  ERC-20 transfer with Privy app signer"
  note "Sign a pathUSD transfer using the Privy server-managed wallet."
  note "stablectl sends the unsigned tx to Privy, which signs and returns it."
  echo ""

  cmd $STABLECTL --chain "$CHAIN" wallet send erc20 \
      --signer privy --wallet-id "$PRIVY_WALLET_ID" \
      --token "$PATH_USD" \
      --to "$WALLET_B" \
      --amount 1.0 \
      --decimals 6
fi

pause

###############################################################################
#  CLEANUP (optional)
###############################################################################

section "Cleanup — logout"
note "Clear the local Privy session."
echo ""
cmd $STABLECTL signer logout

printf "\n${BOLD}${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n"
printf "${BOLD}${GREEN}  Demo complete!${RESET}\n"
printf "${DIM}  Docs:    https://docs.tempo.xyz${RESET}\n"
printf "${DIM}  Faucet:  https://docs.tempo.xyz/quickstart/faucet${RESET}\n"
printf "${BOLD}${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n\n"
