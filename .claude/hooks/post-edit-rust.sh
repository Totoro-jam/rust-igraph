#!/usr/bin/env bash
# PostToolUse hook (Edit/Write): auto-format and lint affected Rust crate.
# Best-effort — never blocks the tool result, only logs issues for the user.

set -euo pipefail

INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE" ] || [[ "$FILE" != *.rs ]]; then
  exit 0
fi

# Single-crate layout: any *.rs in src/ / tests/ / benches/ / examples/
# means the rust-igraph crate. Run fmt + clippy best-effort; never propagate
# errors — Edit already succeeded, this is just a heads-up.
{
  cargo fmt 2>&1 || true
  cargo clippy --quiet --all-targets 2>&1 | tail -5 || true
} >&2

exit 0
