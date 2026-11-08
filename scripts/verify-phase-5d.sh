#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# LendRescue — Phase 5D: Verification Runner
#
# Usage:
#   ./scripts/verify-phase-5d.sh [--offline | --live]
#
# Defaults to --offline if no argument is provided.
# ==============================================================================

MODE="${1:---offline}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

echo "======================================================================"
echo " LendRescue Phase 5D Verification: Mode = ${MODE}"
echo "======================================================================"

SP_PID=""
cleanup() {
    if [ -n "${SP_PID}" ]; then
        echo "--> Stopping background Surfpool process (PID: ${SP_PID})..."
        kill -15 "${SP_PID}" 2>/dev/null || true
        wait "${SP_PID}" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

if [ "${MODE}" = "--live" ]; then
    echo "--> [1/9] Checking / starting Surfpool Mainnet Fork on 127.0.0.1:8899..."
    if ! curl -s -X POST http://127.0.0.1:8899 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}' >/dev/null 2>&1; then
        echo "    Launching surfpool on 127.0.0.1:8899..."
        surfpool start --network mainnet --no-tui --no-studio --no-deploy --port 8899 >/tmp/surfpool-5d.log 2>&1 &
        SP_PID=$!
        
        # Wait up to 15 seconds for local RPC to become responsive
        READY=0
        for i in $(seq 1 15); do
            sleep 1
            if curl -s -X POST http://127.0.0.1:8899 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}' >/dev/null 2>&1; then
                READY=1
                break
            fi
        done
        if [ $READY -ne 1 ]; then
            echo "ERROR: Surfpool failed to start within 15 seconds. Logs:"
            cat /tmp/surfpool-5d.log
            exit 1
        fi
        echo "    Surfpool live and responding on 127.0.0.1:8899."
    else
        echo "    Surfpool already running on 127.0.0.1:8899."
    fi

    echo "--> [2/9] Running Live Protocol Orchestration Tests..."
    SURFPOOL_LIVE=1 cargo test --test protocol_orchestration_tests -- --nocapture

    if [ -n "${SP_PID}" ]; then
        echo "--> Stopping background Surfpool process (PID: ${SP_PID}) to free port 8899 for Anchor..."
        kill -15 "${SP_PID}" 2>/dev/null || true
        wait "${SP_PID}" 2>/dev/null || true
        SP_PID=""
        sleep 1
    fi
else
    echo "--> [1/9] Running Deterministic Offline Protocol Orchestration Tests..."
    cargo test --test protocol_orchestration_tests
fi

echo "--> [3/9] Verifying code formatting (cargo fmt --all --check)..."
cargo fmt --all -- --check

echo "--> [4/9] Running complete workspace tests (cargo test --workspace)..."
cargo test --workspace

echo "--> [5/9] Running Clippy static analysis (cargo clippy --workspace --all-targets -- -D warnings)..."
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "--> [6/9] Building Anchor program (anchor build)..."
anchor build

echo "--> [7/9] Running Anchor integration & security tests (anchor test --skip-build)..."
anchor test --skip-build

echo "--> [8/9] Running Kamino adapter tests (pnpm --filter @lend-rescue/kamino-adapter test)..."
pnpm --filter @lend-rescue/kamino-adapter test

echo "--> [9/9] Checking TypeScript compilation (pnpm exec tsc --noEmit)..."
pnpm exec tsc --noEmit

echo "--> Checking git diff for unstaged trailing whitespace / artifacts..."
git diff --check

echo "======================================================================"
echo " Phase 5D Verification Complete: ALL CHECKS PASSED"
echo "======================================================================"
