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

echo ""
echo "=== All-band distribution ==="
echo ""
echo "13 sine waves (one per band except DC). Each band ~7.7% in Raw."
echo "K-wt attenuates low bands, boosts high bands."
echo "Dyn ~0 for all bands (constant amplitude sine waves)."
echo ""
$B test_data/allband.wav 2>/dev/null | grep -E "^(Raw|K-wt)\(%\)|^Diff |^Dyn\(dB\)"

echo ""
echo "=== Format support ==="
echo ""
echo "AIFF (1kHz):"
$B test_data/1khz.aiff 2>/dev/null | grep "^Raw(%"
echo ""
echo "MP3 (1kHz):"
$B test_data/1khz.mp3 2>/dev/null | grep "^Raw(%"
echo ""
echo "FLAC (1kHz):"
$B test_data/1khz.flac 2>/dev/null | grep "^Raw(%"

echo ""
echo "=== Sample rate warning ==="
echo ""
echo "96kHz file should show K-weight warning:"
$B test_data/1khz_96k.wav 2>&1 | grep -E "(Warning|^Raw\(%)"
