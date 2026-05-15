#!/usr/bin/env bash
# PostToolUse hook (Edit/Write): auto-format and lint affected Rust crate.
# Best-effort — never blocks the tool result, only logs issues for the user.

set -euo pipefail

INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE" ] || [[ "$FILE" != *.rs ]]; then
  exit 0
fi

# Identify affected crate.
case "$FILE" in
  */crates/igraph-core/*)        CRATE="igraph-core" ;;
  */crates/igraph-algorithms/*)  CRATE="igraph-algorithms" ;;
  */crates/igraph/*)             CRATE="igraph" ;;
  *)                             CRATE="" ;;
esac

# Run fmt + clippy on the affected crate (best-effort, do not propagate
# errors — Edit already succeeded; user just gets a heads-up).
{
  if [ -n "$CRATE" ]; then
    cargo fmt --package "$CRATE" 2>&1 || true
    cargo clippy --package "$CRATE" --quiet --all-targets 2>&1 | tail -5 || true
  else
    cargo fmt --all 2>&1 || true
  fi
} >&2

exit 0
