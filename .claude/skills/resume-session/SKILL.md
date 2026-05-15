---
name: resume-session
description: Get back into the project after a part-time break (days to weeks). Re-orients to the current state by reading ALGORITHMS.md, the latest commits, and any wip AWU, then proposes the next concrete step. Use when user says "where were we", "resume", "what was I doing", "let's continue", or after returning from a break.
---

# /resume-session

For the part-time + AI mode this project uses: long gaps are normal. This
skill gets the developer (and the AI context) back to a productive state
in under 5 minutes.

## Workflow

### 1. Sync the working tree

Run silently:

```bash
git status --short
git log --oneline -10
git fetch origin
```

If the working tree is dirty, surface it first — that's the most likely
"something I forgot to commit" pitfall after a break.

### 2. Read the tracking docs in order

Read these in this order, surfacing anything noteworthy:

1. `.codefuse/tracking/ALGORITHMS.md` — find every row with status `wip`
   or `blocked`. Note the date last touched.
2. `.codefuse/tracking/RESUME.md` — manual notes from past sessions
   (gotchas, deferred decisions). Append today's session start.
3. `.codefuse/tracking/RETRO.md` — most recent Phase retro for context on
   current trajectory.
4. The most recent commit on `main` — `git show --stat HEAD`.

### 3. Health check

```bash
cargo test --workspace 2>&1 | tail -5
```

If tests are red on `main`, **stop and investigate**. Something committed
broken — that's the highest-priority fix before any new work.

### 4. Triage open work

For each `wip` AWU older than 4 weeks: ask the user whether to resume,
flip to `todo`, or close. Stale `wip` rots — clear it.

For each `blocked` AWU: re-evaluate the blocker. If the blocker is now
unblocked (e.g. prerequisite AWU landed), flip to `todo`.

### 5. Propose the next concrete step

Based on:
- The Phase the project is in (per ALGORITHMS.md counters)
- AWUs marked `wip` or freshly unblocked
- User's stated time budget

Suggest exactly **one** next AWU and the matching skill to invoke. Format:

> Most recent work: <commit summary>.
> Open: <X> wip, <Y> blocked.
> Suggested next step: `/awu-<step> ALGO-XXX-NNN` — <one-line reason>.

### 6. Append a session marker

Add to `.codefuse/tracking/RESUME.md`:

```
## <ISO date> — resumed

- Last commit: <sha> — <title>
- wip AWUs reviewed: <list>
- Decisions made: <bullets>
- Picked up: ALGO-XXX-NNN via `/<skill>`
```

## Anti-patterns

- **Do NOT propose multiple parallel AWUs** unless the user explicitly
  asks. Part-time mode is one-thing-at-a-time.
- **Do NOT silently fix red tests.** Surface them; the user may want to
  understand the regression first.
- **Do NOT skip reading RESUME.md.** That's where past-you left clues for
  present-you.
