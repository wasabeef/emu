#!/bin/bash

# Prevent commits with WIP in the message (except when using --no-verify)
if grep -qE "^WIP" "$1"; then
    echo "Error: Commit message starts with 'WIP'"
    echo "If you really want to commit WIP, use --no-verify flag"
    exit 1
fi