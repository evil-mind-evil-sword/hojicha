#!/bin/bash

# Script to debug visual.rs with full logging

echo "Starting visual debug session..."
echo "Logs will be written to: /tmp/visual_debug.log"
echo ""

# Clear old logs
> /tmp/visual_debug.log

# Run the logged version
echo "Running visual_logged example..."
echo "Press any keys to test, 'q' to quit"
echo ""
cargo run --example visual_logged

echo ""
echo "Program exited. Log contents:"
echo "=================================="
cat /tmp/visual_debug.log
echo "=================================="
echo ""
echo "Look for:"
echo "  - 'Key event' lines to see what keys are received"
echo "  - 'QUIT key pressed' if quit is triggered unexpectedly"
echo "  - 'is_quit: true' if Cmd::none() is returning quit (shouldn't happen)"