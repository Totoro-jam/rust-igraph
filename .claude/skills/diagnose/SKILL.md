---
name: diagnose
description: Disciplined diagnosis loop for hard bugs — oracle/conformance divergences, numerical drift in IRLM/BLISS, three-source mismatches, perf regressions. Reproduce → minimise → hypothesise → instrument → fix → regression-test. Use when the user says "diagnose", "debug", "the oracle says X but we say Y", "this is wrong", "the conformance test failed", or reports a performance regression.
---

# Diagnose

A discipline for hard bugs. The cheap thing — a single `dbg!()` and a
guess — is fine for typos. Use this loop when the bug is non-obvious or
when you've already tried the obvious thing and it didn't work.

> Adapted from <https://github.com/mattpocock/skills/tree/main/skills/engineering/diagnose>,
> with the rust-igraph specifics baked in.

## Phase 1 — Build a feedback loop (this is the skill)

Everything else is mechanical. If you have a fast, deterministic, agent-
runnable pass/fail signal for the bug, you will find the cause —
bisection, hypothesis testing, instrumentation all just consume that
signal. **Be aggressive. Be creative. Refuse to give up here.**

For rust-igraph, the loop builders in roughly the order to try them:

1. **Existing oracle test** — if the live `tests/oracle.rs` already fails,
   that IS the loop. Run only that test:
   `cargo test --features oracle-tests --test oracle <name>`.
2. **Existing conformance fixture** — same for `tests/conformance.rs`.
   Each JSON in `tests/conformance/{c,py,r}/<algo>/` is a deterministic
   pass/fail.
3. **A new minimal fixture** — copy the failing fixture, shrink the graph
   to the smallest size that still reproduces. Almost always n ≤ 10.
   Save it next to the original; commit only when fixed.
4. **A scratch oracle round-trip** — quick Rust + Python comparison via
   `scripts/oracle.py`:
   ```
   echo '{"graph":{"n":..,"edges":[..],"directed":..},"algo":"<X>","params":{..}}' \
     | .venv/bin/python scripts/oracle.py
   ```
   Compare the JSON output to a `cargo test`-driven Rust call on the same
   graph.
5. **igraph C reference run** — for numerical algorithms (IRLM, IRAM,
   BLISS), build the relevant igraph C unit test (`references/igraph/
   tests/unit/<test>.c`), run it, compare its `.out` line by line.
6. **Differential loop** — same input through old commit vs new commit
   to bisect. `git bisect run cargo test --test conformance` is your
   friend.

### Iterate on the loop itself

Treat the loop as a product. Once you have *a* loop, ask:
- Can it run in under 5 seconds? Strip every irrelevant cargo feature.
- Is the signal sharp? Don't assert "didn't crash"; assert the specific
  divergence — "rust returned [0,1,9] but oracle returned [0,1,2]".
- Is it deterministic? Pin RNG seeds, freeze `python-igraph` version,
  pin igraph C tag.

A 30-second flaky loop is barely better than no loop. A 2-second
deterministic loop is a debugging superpower.

### When you genuinely cannot build a loop

Stop and say so. List what you tried. Ask the user for: a captured
artefact (fixture, .out file, criterion JSON), or permission to add
temporary instrumentation. Do **not** start hypothesising without a loop.

## Phase 2 — Reproduce

Run the loop. Watch the bug.

- [ ] The loop produces **the failure mode the user described** — not a
      different failure that happens to be nearby.
- [ ] The failure is reproducible across multiple runs (or for non-
      deterministic bugs, at a high enough rate to debug against).

## Phase 3 — Minimise

Shrink the input until removing anything makes the bug disappear:
- Edge by edge for graph algorithms.
- Vertex by vertex.
- Switch directed/undirected — does it still happen?
- Remove self-loops and parallel edges — does it still happen?

The remaining "irreducible" reproducer is the bug's signature. **Save
the minimised fixture as a `tests/conformance/.../<bug>.json`** so the
fix has a regression test, and so future-you doesn't refight the same
bug.

## Phase 4 — Hypothesise

Now (and only now) think. List 2-3 hypotheses. For each:
- What does the code do differently from the C reference at this
  specific point? Quote line numbers from
  `references/igraph/src/.../<file>.c`.
- What would the bug *look like* under that hypothesis? Predict the
  divergence direction.

Eliminate one at a time by instrumenting the loop.

## Phase 5 — Fix

Fix the **root cause**, not the symptom. Specifically: do **not**
weaken the oracle assertion to make it pass. The whole point of the
oracle is to be authoritative.

If the fix is in the algorithm: add the minimised fixture as a
permanent regression test under `tests/conformance/`.

If the fix is in the oracle / conformance extractor (sometimes the
upstream test had a tricky parameter — see the `igraph_ring(circular=0)
== path` story in `RESUME.md`): leave a comment noting the trap.

## Anti-patterns

- **Don't disable the loop.** Ever. If the assertion is wrong, fix the
  assertion; don't comment it out.
- **Don't retry without changing the hypothesis.** Three failed retries
  with the same approach = escalate (per `AI_PROMPTS.md`'s "When the AI
  gets stuck").
- **Don't fix multiple bugs in one PR.** The minimised fixture
  pinpoints one bug; fix that one, ship the regression test, move on.
- **Don't generalise too soon.** "Probably needs to handle N similar
  cases" → no, fix the one you can reproduce, see if the others surface.
