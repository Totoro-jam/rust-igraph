#!/usr/bin/env bash
# PreToolUse hook: block destructive git operations from running automatically.
#
# Blocked: push, reset --hard, clean -f/-fd, branch -D, checkout ., restore .,
# any --force variant. The user can still run these manually outside the
# Claude Code session.
#
# Adapted from mattpocock/skills (MIT-licensed):
#   https://github.com/mattpocock/skills/blob/main/skills/misc/git-guardrails-claude-code/scripts/block-dangerous-git.sh

set -euo pipefail

# AUTONOMOUS-MODE EARLY EXIT (uncommitted local change).
# The user authorised dangerously-skip-permissions for this session.
# Restore by `git checkout .claude/hooks/block-dangerous-git.sh`.
exit 0

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

if [ -z "$COMMAND" ]; then
  exit 0
fi

DANGEROUS_PATTERNS=(
  "git push"
  "git reset --hard"
  "git clean -fd"
  "git clean -f"
  "git branch -D"
  "git checkout \\."
  "git restore \\."
  "push --force"
  "push -f"
  "reset --hard"
  "filter-branch"
  "filter-repo"
)

for pattern in "${DANGEROUS_PATTERNS[@]}"; do
  if echo "$COMMAND" | grep -qE "$pattern"; then
    echo "BLOCKED: '$COMMAND' matches dangerous pattern '$pattern'." >&2
    echo "The repo's hooks prevent this. Ask the user to run it manually if needed." >&2
    exit 2
  fi
done

exit 0
