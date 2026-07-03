---
title: Ralph-loop iteration launches return "failed" exit-1 despite successful completion
type: notes
date: 2026-05-07
status: active
tags: [ralph-loop, bug, ci-cd, claude-code, skill]
---

# Ralph-loop iteration launches return "failed" exit-1 despite successful completion

## Symptom

Every `Bash run_in_background: true` invocation of `~/.claude/skills/ralph-loop/scripts/run-iteration.sh` ends with the Claude Code task notification reporting:

```
status: failed
summary: Background command "..." failed with exit code 1
```

…even though the iteration's actual work (Phase 1 implement → Phase 2 review/merge → cmux pane teardown) all succeeded.

Observed across iter-15, iter-16, iter-17 (in this session). All three merged cleanly to `origin/main` with no review issues.

## Evidence the iteration actually succeeded

For each affected iteration, three independent signals confirm success:

1. **Script-side sentinel** — `~/.cache/ralph-loop/hoppy/iter-<N>-done` exists with body:
    ```
    2026-05-07T16:31:31Z
    exit_code=0
    ```

    Written by `on_exit() { local code=$?; ... case "$code" in 0) kind="done" ;; ... }` (run-iteration.sh:20–36). The trap captures `$?` at script-exit time. `exit_code=0` means the script's `exit 0` line at run-iteration.sh:329 was reached.

2. **Stdout log** — `iter-<N>.log` ends with the literal string `"Iteration <N> completed successfully."` followed by two `OK` lines from `cmux clear-progress` and `cmux clear-status`. The script's tail (lines 308–329) is:
    ```bash
    cmux close-surface --surface "$SURFACE_ID" 2>/dev/null || true
    echo "Iteration ${ITER_NUM} completed successfully."
    sleep 5
    cmux clear-progress "${WS_FLAG[@]}" 2>/dev/null || true
    cmux clear-status   "$STATUS_KEY" "${WS_FLAG[@]}" 2>/dev/null || true
    exit 0
    ```

    Both cmux teardown calls are `|| true`, so they can't propagate a non-zero exit.

3. **Git state** — the merge commit for `iter-<N>/...` lands on `origin/main`. iter-15 → `c77b1eb`, iter-16 → `48bbdc4`, iter-17 → `ca8655c`.

So the script *itself* exits 0. The wrapper around it reports 1.

## Hypotheses (ranked)

1. **Most likely — terminal/pane teardown propagating a signal to the wrapper.**
    The script closes its cmux pane (`cmux close-surface --surface "$SURFACE_ID"`) just before exiting. The pane was the script's controlling terminal. If closing it causes the parent shell (the Bash-tool wrapper that ran `script > iter-N.log 2>&1`) to receive SIGHUP, the wrapper may exit with a non-zero status that Claude Code's tool reports as `1`. The script's own EXIT trap fires before this, capturing `$?=0` correctly into the sentinel — so the sentinel and the wrapper exit code disagree.

2. **Less likely — Bash tool reading the wrapper's exit code from a stale source.**
    The Claude Code Bash-tool's background mode may capture the first non-zero exit it sees from any stage of its wrapper construction (env-var assignment, redirection, the actual command). If any sub-step transiently exits non-zero (e.g. a shell-builtin returning non-zero on a side-effect), it could shadow the script's eventual `exit 0`.

3. **Unlikely — race between EXIT trap writing the sentinel and the wrapper observing the exit.**
    The trap runs *after* `exit 0` has set `$?=0`, so the sentinel is correct. The wrapper's reported exit code is independent.

## Workaround in current use

The orchestrator already does the right thing: **trust the sentinel, not the wrapper exit code**. After each `failed` notification, the next loop tick checks `~/.cache/ralph-loop/hoppy/iter-<N>-done` and proceeds to VERIFY+CLEANUP+ADVANCE if the sentinel says `exit_code=0`. The merge is independently verified via `git log origin/main` matching `iter[-/]?N`.

This means the false-failed exits are noisy but not harmful — they just produce a misleading task-notification.

## Possible fixes (in the skill, when there's appetite)

- **`disown` after launching, then `wait` separately** — separates the cmux session from the wrapper exit. But complicates exit-code capture.
- **Detach the cmux session-close from the script** — write the sentinel, exit 0 *first*, run the cmux teardown asynchronously via `(cmux close-surface ...) &`. The wrapper sees `exit 0` immediately; the cleanup runs in a child that nobody waits on.
- **Make the wrapper trust the sentinel** — change the launch invocation to:
    ```bash
    bash -c '<launch>; cat <cache>/iter-<N>-done >/dev/null && exit 0 || exit 1'
    ```

    so the wrapper's exit code reflects the sentinel rather than the script's exit. Slightly hacky but eliminates the false-failed signal.
- **Trap SIGHUP in the script** to swallow the signal before the wrapper sees it. Worth trying if hypothesis 1 is correct.

## Validation that hypothesis 1 is correct (TODO)

Reproduce in isolation:

```bash
# Minimal repro — script that opens a cmux pane, closes it, then exits 0
$ bash -c 'cmux new-pane --direction right; sleep 1; cmux close-surface --surface ...; exit 0; echo $?' &
$ wait $!; echo "wrapper exit: $?"
```

If the wrapper exit is `1` even though `exit 0` ran, hypothesis 1 is confirmed and the fix lies in the cmux teardown timing.

## Pending fixes — batch when next touching the skill

When the next maintenance pass on `~/.claude/skills/ralph-loop/scripts/run-iteration.sh` happens (e.g., to fix the false-failed exits above), bundle these in the same PR:

### 1. Move Copilot-review trigger from Phase 2 start → Phase 1 end

**Today:** `PROMPT_IMPLEMENT` (run-iteration.sh:66) ends with step `6) /create-pr`. `PROMPT_REVIEW` (run-iteration.sh:69) starts with step `1) /review-pr and fix all review issues`. Inside `/review-pr` is where GitHub Copilot is triggered (and CodeRabbit, etc.). So Copilot starts only when Phase 2 begins, and Phase 2 then has to wait for Copilot to produce comments before fixing them.

**Change:** trigger Copilot's PR review at the *end* of Phase 1, immediately after `/create-pr`. Phase 2's `/review-pr` then starts with the Copilot review already in progress (or already complete), so phase 2's wait time drops.

**Concretely:** modify `PROMPT_IMPLEMENT` so step 6 becomes `6) /create-pr  7) Trigger automated reviews (Copilot, CodeRabbit) on the PR — do NOT wait for them; just kick them off then exit.` The actual trigger command depends on how `/review-pr` does it today; replicate the trigger logic without the wait/fix loop. Likely either `gh pr edit <num> --add-reviewer github-copilot[bot]` or whatever the `/review-pr` skill does internally.

**Why this is safe:** Phase 2 already has `/review-pr` which will discover any reviews already posted (and any still in flight). Pre-triggering only buys parallelism — it doesn't change the semantics of Phase 2.

**Risk:** if the trigger is async-fire-and-forget but a network issue prevents it, Phase 2's `/review-pr` will trigger again redundantly. Tolerable — Copilot is idempotent on a single PR.

### 2. False-failed exits

Fix per the hypotheses + suggested fixes above. Most-promising single change: run cmux teardown asynchronously so the wrapper sees `exit 0` *before* the pane-close-induced SIGHUP can propagate:

```bash
echo "Iteration ${ITER_NUM} completed successfully."
( sleep 5
  cmux clear-progress "${WS_FLAG[@]}" 2>/dev/null || true
  cmux clear-status "$STATUS_KEY" "${WS_FLAG[@]}" 2>/dev/null || true ) &
disown
exit 0
```

Or, even simpler, swallow SIGHUP at script start: `trap '' HUP`.

## Cross-references

- Skill source: `~/.claude/skills/ralph-loop/scripts/run-iteration.sh`
- Cache: `~/.cache/ralph-loop/hoppy/`
- Affected iterations this session: 15 (`c77b1eb`), 16 (`48bbdc4`), 17 (`ca8655c`); iter-18 in flight at time of writing
- Related: ralph-loop state-machine model relies on disk sentinels precisely because the wrapper exit code is unreliable — the disk file is the source of truth.
