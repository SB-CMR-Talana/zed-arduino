#!/bin/bash
# Script to check Arduino extension debug logs

echo "=== Arduino Extension Debug Log Checker ==="
echo ""
echo "1. Checking Zed LSP logs (extension stderr/eprintln output):"
echo "   Run: journalctl --user -u zed -n 200 --no-pager | grep -i arduino"
echo "   OR check Zed's output panel: Cmd+Shift+P -> 'zed: open log'"
echo ""
echo "2. Checking sketch detection debug logs:"
echo ""

WORKSPACE="/home/sigrid/code/esp/indev"

if [ -f "$WORKSPACE/zed-sketch-detection.log" ]; then
    echo "✓ Found workspace log:"
    cat "$WORKSPACE/zed-sketch-detection.log"
else
    echo "✗ No log at: $WORKSPACE/zed-sketch-detection.log"
fi

echo ""

TEMP_LOG="/tmp/zed-sketch-detection-_home_sigrid_code_esp_indev.log"
if [ -f "$TEMP_LOG" ]; then
    echo "✓ Found temp log:"
    cat "$TEMP_LOG"
else
    echo "✗ No log at: $TEMP_LOG"
fi

echo ""
echo "3. Checking arduino debug log:"
if [ -f "$WORKSPACE/zed-arduino-debug.log" ]; then
    echo "✓ Found:"
    cat "$WORKSPACE/zed-arduino-debug.log"
else
    echo "✗ No log at: $WORKSPACE/zed-arduino-debug.log"
fi

if [ -f "/tmp/zed-arduino-debug.log" ]; then
    echo "✓ Found:"
    cat "/tmp/zed-arduino-debug.log"
else
    echo "✗ No log at: /tmp/zed-arduino-debug.log"
fi

echo ""
echo "4. Checking Language Server logs:"
if [ -f "/tmp/inols-err.log" ]; then
    echo "✓ Last 30 lines of LS error log:"
    tail -30 /tmp/inols-err.log
else
    echo "✗ No LS error log"
fi

echo ""
echo "=== To see eprintln output from extension ==="
echo "Run Zed from terminal: zed /home/sigrid/code/esp/indev"
echo "All eprintln! output will appear in the terminal"
