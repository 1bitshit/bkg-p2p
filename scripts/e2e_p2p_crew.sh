#!/usr/bin/env bash
# Multi-process smoke: three `bkg-peer serve` nodes on loopback with distinct data dirs.
# Requires: built `target/release/bkg-peer` (or set BKG_PEER_BIN).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BKG_PEER_BIN:-$ROOT/target/release/bkg-peer}"
if [[ ! -x "$BIN" ]]; then
  echo "Build first: cargo build --release  (expected $BIN)" >&2
  exit 1
fi
BASE="/tmp/bkg-peer_e2e_$$"
mkdir -p "$BASE"/{a,b,c}
cleanup() { rm -rf "$BASE"; }
trap cleanup EXIT

# Node A: web 8081, bootstrap none (listener)
BKG_PEERD_HOME="$BASE/a" "$BIN" serve --web 127.0.0.1:8081 --listen /ip4/127.0.0.1/tcp/4001 &
PID_A=$!
sleep 2
BKG_PEERD_HOME="$BASE/b" "$BIN" serve --web 127.0.0.1:8082 --listen /ip4/127.0.0.1/tcp/4002 \
  --bootstrap /ip4/127.0.0.1/tcp/4001 &
PID_B=$!
BKG_PEERD_HOME="$BASE/c" "$BIN" serve --web 127.0.0.1:8083 --listen /ip4/127.0.0.1/tcp/4003 \
  --bootstrap /ip4/127.0.0.1/tcp/4001 --crew-worker &
PID_C=$!
sleep 4

curl -sf "http://127.0.0.1:8081/api/status" | head -c 200 || true
echo
curl -sf "http://127.0.0.1:8082/api/a2a/peers" | head -c 200 || true
echo

kill $PID_C $PID_B $PID_A 2>/dev/null || true
wait $PID_C 2>/dev/null || true
wait $PID_B 2>/dev/null || true
wait $PID_A 2>/dev/null || true
echo "e2e_p2p_crew: done (manual: check logs for gossip + crew worker)"
