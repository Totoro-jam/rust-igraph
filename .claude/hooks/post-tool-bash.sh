#!/usr/bin/env bash
# PostToolUse hook (Bash): append every command we ran to a session log so
# future `/resume-session` can audit what AI actually did.
#
# The log itself is gitignored (.codefuse/tracking/agent_actions.log).

set -euo pipefail

LOG=".codefuse/tracking/agent_actions.log"
mkdir -p "$(dirname "$LOG")"

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if [ -n "$COMMAND" ]; then
  printf "%s\t%s\n" "$TIMESTAMP" "$COMMAND" >> "$LOG"
fi

exit 0
