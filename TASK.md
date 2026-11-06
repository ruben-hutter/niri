# TASK — integrating `place` into niri

Scratch context for this branch (`notes/place-integration`). Full plan:
[`notes/place-integration/PLAN.md`](notes/place-integration/PLAN.md).

> **Branch warning:** research notes only, co-written with an LLM. niri's
> CONTRIBUTING.md forbids LLM-written upstream content — never open PRs,
> issues, or comments from this branch, and rewrite everything upstream-
> facing in your own words.

## Goal

Bring the `place` workflow (`~/.config/niri/place/`, workspace arrangement by
monitor role across setup changes) into niri proper — staged, upstreamable.

## Status (2026-11-06)

**Strategy pivot (2026-11-06):** no Matrix post after all. Build the part
that "fits" niri without pre-discussion (Part 1 roles) and submit the
persisted-setups part as a separate PR, accepting it may be rejected.

**PR1 implemented** on `feat/output-roles` (branched off fresh
upstream/main @ 2c0b311b), commit `4819b69f`, awaiting ruben's review:

- `output { role N }` — lower role = more important; lowest connected
  role becomes `primary_idx` (ties broken by config order, like
  focus-at-startup). No roles configured → behavior byte-identical
  (primary_idx was provably always 0).
- Effects (inherited from existing primary_idx uses): orphaned-workspace
  fallback, startup focus default (focus-at-startup still wins).
- Implementation: `Layout` stores `niri_config::Outputs` and resolves
  roles itself (set in `new`/`update_config`, recomputed on add/remove
  output) — no `add_output`/`Monitor::new` signature changes. New
  `Outputs::find_with_position()` in niri-config powers the config-order
  tiebreak in both layout and niri.rs.
- Tests: 5 in `src/layout/tests.rs` + 1 in `src/tests/output_role.rs`
  + extended config parse snapshot; wiki `### role` section added.
  254/254 niri, 38/38 niri-config, 3/3 niri-ipc green; fmt+clippy clean
  (remaining clippy warnings are pre-existing upstream).
- Scope decisions (2026-11-06 session): PR1 changes ONLY primary_idx
  semantics; wizard dropped entirely for now; PR2 = arrangement engine
  + opt-in persisted setups; ruben reviews/rewrites all code before any
  PR (he owns it per the LLM policy tension — his call).

Next: ruben reviews PR1. Before PR2 code: present concrete scenario
examples for the trigger-surface decision (when does arrangement
redistribution kick in), per ruben's request.

## Decisions so far

1. **Split the feature in two parts** (ruben's call, verified against code):
   - **Part 1 — roles in config (declarative).** niri already has a
     `primary_idx` (`src/layout/mod.rs:385`, hardcoded `0` at `:871`,
     fallback destination at `:926`) that users cannot configure. Expose an
     importance order (primary, 2nd, n-th) on outputs. Uncontroversial,
     fills a real gap. No wizard needed. No state on disk.
   - **Part 2 — setup recognition + click-to-define (interactive).** For
     ad-hoc setups (once-only TV, hotel docks): detect an unconfigured
     arrangement and let the user click desk order + roles on the monitors
     (badge wizard). Preferred shape to propose: **wizard-as-config-editor**
     — the clicks write what the user would have written into the config
     (niri hot-reloads config), so the result stays declarative and
     reproducible. No invisible state.
   - **Part 3 — learned splits (optional, last).** The only truly learned
     piece: remembering per-setup workspace distribution (block sizes).
     Behind an opt-in config section writing to `~/.local/state/niri/`,
     or dropped entirely if upstream says no. place.py proves the value;
     upstream decides.
2. **Positioning (ruben, for the Matrix post):** pro-reproducibility, on the
   maintainer's side. Known setups belong in config. Clicking is only for
   *new* setups (defining a once-used setup by hand in config wastes time).
   Roles are niri's genuinely missing primitive. Opt-in persistence =
   user consent, cache-not-authority (delete file → vanilla).
3. **Do NOT cite "Windows has it" as the main argument** — niri-native
   framing is stronger: niri already keeps invisible volatile state
   (`original_outputs`, `last_active_workspace_id`); persistence adds no new
   kind of state, only durability. Evidence the volatile heuristics fail:
   issue #4702.

## The one question to ask upstream

"Would learned arrangement state in XDG_STATE_HOME behind an opt-in config
section be acceptable — or only config-declared (wizard writes config)?"
Get this verdict before writing any feature code.

## Update 2026-11-03 — place.py incident informs the design

place.py mis-placed a workspace (Zen) at the work setup: a stale
*count-based* split (`[2,2,0]`, drifted on 10-29 by an incidental workspace
and mirrored into the setup's memory while settled) hijacked the arrival
and overrode the correct `[1,2,0]` cut. Analyzed and replayed red/green
against `layout.py` (replay script: /tmp/place-debug/replay_arrival.py);
state remediated manually (work split = `[1,2,0]`).

Decision:
- **No model rework in place.py** (the id-blocks fix is subsumed by the
  niri integration; don't build it twice). place.py stays the daily driver
  until the integration lands. The drift-guard stopgap **was implemented**
  on 2026-11-03: `layout.arrive()` distrusts splits whose sum exceeds the
  open workspaces (place.py logs when it fires); 3 regression tests added
  to test_layout.py (suite 52/52 green).
- The lessons are binding for `arrangement.rs` / upstream design:
  see PLAN.md §6.7 (identity-based memory, death propagation,
  last-known-role seeding).
- The incident is also empirical evidence for Part 1: what the user
  experiences is *role affinity* ("Zen belongs on the 2nd screen"),
  never a strip cut position.

## Pointers

- place sources: `~/.config/niri/place/` (layout.py = pure state machine +
  408-line test suite, ports 1:1 to Rust).
- niri hooks: `Layout::add_output/remove_output` (src/layout/mod.rs:760/878),
  `Niri::reposition_outputs` (src/niri.rs:2955),
  `move_workspace_to_output_by_id` (src/layout/mod.rs:3482),
  `find_preferred_output` (src/layout/workspace.rs:479),
  `last_active_workspace_id` (src/layout/mod.rs:358), `is_locked()`.
- Upstream links: PR #4318 (relative output placement — closed 2026-10-27
  *per LLM policy*, no design verdict; discussion #3793 still open),
  issue #4702 (identical monitors → wrong monitor; identity-ambiguity).
- Repo facts: upstream remote now `niri-wm/niri` (repo moved org; fork
  network followed automatically — `ruben-hutter/niri` parent verified).
  Fork is ~37 commits behind upstream/main.
- Full analysis: notes/place-integration/PLAN.md.

## Repo hygiene

This repo is a plain checkout (no wtree). Do feature work for real code on
separate `feat/*` branches; keep `notes/place-integration` notes-only.
