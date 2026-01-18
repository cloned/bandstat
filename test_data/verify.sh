#!/bin/bash
# Verify bandstat produces correct results for known inputs.
set -e

cd "$(dirname "$0")/.."
~/.cargo/bin/cargo build --release --bin bandstat 2>/dev/null

B=./target/release/bandstat

echo "=== Band classification ==="
echo ""
echo "100Hz -> BASS:"
$B test_data/100hz.wav 2>/dev/null | grep "^Raw(%"
echo ""
echo "1kHz -> UMID:"
$B test_data/1khz.wav 2>/dev/null | grep "^Raw(%"
echo ""
echo "5kHz -> PRES:"
$B test_data/5khz.wav 2>/dev/null | grep "^Raw(%"

echo ""
echo "=== K-weighting ==="
echo ""
echo "1kHz (no change):"
$B test_data/1khz.wav 2>/dev/null | grep -E "^(Raw|K-wt)\(%\)"
echo ""
echo "100Hz + 3kHz (low attenuated, high boosted):"
$B test_data/mix_100_3000hz.wav 2>/dev/null | grep -E "^(Raw|K-wt)\(%\)"
