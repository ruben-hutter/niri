# Integrating `place` into niri — research & plan

**Status: research only. No code changed.**
Written 2026-10-31 against upstream niri @ 2026-10-27; fork `ruben-hutter/niri`
~37 commits behind upstream/main. Companion scratch file: `/TASK.md` (branch root).

---

## 1. Executive summary

`place` (~430 LoC Python + ~410 LoC tests in `~/.config/niri/place/`) keeps
niri workspaces arranged by *monitor role* across monitor-setup changes.
Integration into niri is feasible; the substrate largely exists
(add/remove-output hooks, OutputName identity model, per-workspace original
outputs, internal primary_idx). Three obstacles, in order of severity:

1. **Upstream policy.** niri's CONTRIBUTING.md forbids LLM-written PRs,
   issues, comments. *Actively enforced:* PR #4318 was closed on 2026-10-27
   citing it. All upstream-facing content must be human-written.
2. **Philosophy.** place runs on learned, invisible, persisted state; niri
   persists nothing and its design-principles doc is wary of invisible
   state. The staging below is built to make this question *small and
   optional* (Parts 1–2 need no learned state at all).
3. **Scope.** The badge wizard + auto-registration are the most
   desktop-environment-flavored bits; mitigated by the wizard-as-config-
   editor reframing (§6).

## 2. What place is

| file | LoC | role |
|---|---|---|
| `layout.py` | 300 | pure state machine: strip (`order`), active cut (`sizes`), per-setup cuts (`splits`), lifecycle verbs (`migrate/transition/arrive/settle/rekey/learn`), `plan()` → moves |
| `place.py` | 560 | daemon: 5 s poll, setup detect/register, converge loop, journalctl lock watch, wizard driver, align, pause/resume |
| `niri.py` | 60 | IPC seam (`niri msg` + Move/Focus raw actions) |
| `arrange_gtk.py` | 167 | badge overlay: click monitors to define desk order / roles |
| `test_layout.py` | 408 | unittest suite of the pure logic |

Model: workspaces form one canonical strip cut into blocks by monitor *role*
(importance order). A *setup* = monitor set + human-declared desk order,
hashed to an id. Each setup remembers its *split* (block sizes); arriving
re-cuts the strip so returning to a desk re-creates its arrangement.

Key invariants worth keeping in any port:
- Same monitor set, different desk arrangement = different setup.
- Never learn during a transition (only while settled).
- Human-declared desk order beats niri's auto-layout guess.
- Monitor-*set* fallback match when arrangement hash differs (drift).

## 3. What niri already has (verified)

| place concept | niri counterpart |
|---|---|
| identity = make model serial | `OutputName` (`niri-config/src/output.rs:119`); workspaces store `original_outputs: Vec<OutputId>` normalized to make/model/serial (`src/layout/workspace.rs`) |
| setup change detection | events: `Layout::add_output/remove_output` (`src/layout/mod.rs:760,878`) via `Niri::add_output/remove_output` (`src/niri.rs:3078,3176`) |
| converge loop | direct synchronous calls: `move_workspace_to_output_by_id` (`src/layout/mod.rs:3482`), `Monitor::move_workspace_to_idx` (`src/layout/monitor.rs:1295`); no IPC, no drop-while-locked |
| **primary monitor** | `MonitorSet.primary_idx` (`src/layout/mod.rs:385`) — **hardcoded 0** (`:871`); orphaned workspaces fall back to it (`:926`, `niri.rs:2974`). *No config field, no action can set it.* ← **the roles gap** |
| focus restore | `last_active_workspace_id` per connector (`src/layout/mod.rs:358`); `focus-at-startup` per output exists (`niri-config/src/output.rs:71`) |
| lock watch | `Niri::is_locked()` internal |
| align | `Niri::reposition_outputs()` (`src/niri.rs:2955`): sorts (config-position first, then by OutputName), overlap-fallback; IPC "set output position" exists (`niri-ipc/src/lib.rs:1090`) |
| wizard UI substrate | in-compositor UI: `src/ui/` (exit dialog, screenshot UI, hotkey overlay); `pick-window` exists internally |
| persistence | **nothing** — niri writes no state files |

Built-in behavior place competes with: per-workspace affinity
(`find_preferred_output`, `src/layout/workspace.rs:479`); named-workspace
`open-on-output` pins with fallback lists (PR #4618). Gap = niri has no
role-block distribution and no persistence; after reboot dynamic workspaces
pile onto one monitor. Issue #4702 (identical monitors → workspaces on wrong
monitor after power-cycle) demonstrates the affinity heuristics' identity
ambiguity.

## 4. What integration removes

Poll loop, converge-retry, journalctl lock-watch, session token, the
pick-window title hack, `niri.py` — all evaporate in-compositor. What
remains: the pure state machine, persistence (optional), wizard UI, align,
role heuristics.

## 5. THE SPLIT (current direction, decided 2026-10-31)

Split place into independent parts with separate upstream conversations:

### Part 1 — Roles in config (declarative) ⭐ most significant contribution
Add an importance order for outputs (primary, 2nd, … n-th) to config:
```kdl
output "DP-3" {
    role 1   // naming TBD: role / priority / importance
}
```
Effects: orphaned-workspace destination (existing `primary_idx` fallback),
focus-at-startup default, new-window target, and — combined with place's
strip-cut logic — deterministic workspace distribution by role blocks.
Fully declarative; no wizard; no on-disk state; slots into an existing
internal concept that is currently unconfigurable. This is the PR most
likely to be accepted and is valuable even if nothing else lands.
(Also worth checking upstream appetite for extending it to relative output
placement — discussion #3793 is still open.)

### Part 2 — Setup recognition + click-to-define (interactive)
Detect an arrangement with no config (new setup) → offer the badge wizard
(2 passes: desk order left→right, then importance) → **wizard-as-config-
editor**: the clicks produce the config the user would have written
(output blocks with roles/positions); niri hot-reloads. Result stays
declarative and reproducible — "clicking" is an input method for config,
not a replacement for it. Covers ruben's use case: known setups live in
config; once-only setups (TV, hotel dock) get defined by two clicks
instead of hand-editing KDL. Implementation can be in-compositor UI or —
simpler first step — the wizard writes/suggests a config snippet.

### Part 3 — Learned splits (optional; the only "magic")
Remember per-setup workspace distribution (place's `splits`): opt-in config
section; state file in `~/.local/state/niri/arrangement.json` (atomic
writes, delete = vanilla); documented consent. Pure state machine port
(`layout.py` → `src/layout/arrangement.rs`) + ported tests. Only this part
needs the upstream verdict below.

**The one question to ask upstream first:** "Would learned arrangement state
in XDG_STATE_HOME behind an opt-in section be acceptable — or only
config-declared (Part 2's wizard-writes-config shape)?" Ask on Matrix /
Discussion *before* writing feature code.

### Argument kit for the persistence question (Part 3)
- Not privacy (XDG_STATE_HOME is user-owned; niri already writes
  screenshots). The real upstream concerns: config-as-single-source-of-
  truth (history-dependent behavior), surprise surviving reboots, forever
  maintenance of a persisted format, precedent. Address, don't dodge.
- Opt-in = consent; absent section = byte-identical behavior (matches the
  "disabled features cost nothing" principle).
- Cache-not-authority: corrupt/stale file → ignore + rebuild (place.py
  already behaves this way); delete file → vanilla.
- niri already keeps invisible *volatile* state (`original_outputs`,
  `last_active_workspace_id`); durability is a difference of degree, not
  kind. Issue #4702 shows the volatile heuristics failing.
- Precedents: i3 restart re-serializes layout; Windows 11 "remember window
  locations per monitor connection" (market validation — mention lightly).

## 6. Design notes (Parts 2–3 implementation)

- Module shape: `src/layout/arrangement.rs` (pure, `State` in/out, the
  split/sizes mirroring invariant in exactly one place — `layout.py`'s
  docstring is already the Rust module spec); hooks in
  `Layout::add_output/remove_output`; `reposition_outputs` derives
  positions from declared desk order (subsumes `place align`; read closed
  PR #4318 first); `src/ui/setup_wizard.rs` for badges; tests port to
  `src/layout/tests.rs` + `src/tests/` (mock outputs exist).
- Config surface: opt-in section only for Part 3. IPC: actions
  (wizard/align/pause), event `ArrangementChanged` for bars — niri itself
  stays out of the notification business.
- Identical monitors (#4702): identities collide (niri "unnames"
  duplicates, `src/backend/tty.rs:1067`). Decide: connector tiebreak,
  declared-order disambiguation, or documented ambiguity.
- Restart semantics: place's way (persist splits only; re-adopt open strip
  on start; cut on first expanded arrival) — needs no new identity scheme.

## 7. Upstream process

- Discuss before code; Matrix #niri:matrix.org or Discussions; frame around
  the problem (multi-monitor distribution across setup changes; #4702),
  not "port my tool".
- LLM policy is enforced (#4318 closure). PRs focused and small (~200 LoC
  reviewable units; cf. merged #4618). A monolithic wizard PR will not land.
- Order of conversations: (1) roles-in-config (Part 1) — concrete gap,
  existing internal concept; (2) wizard shape (Part 2) — after 1 gains
  traction; (3) persistence verdict (Part 3) — the explicit question.

## 8. Repo facts

- Upstream moved org: canonical repo is `niri-wm/niri` (YaLTeR redirects).
  Local `upstream` remote updated; fork network followed automatically
  (`ruben-hutter/niri` parent = `niri-wm/niri`, verified via API) — no
  re-fork needed. This branch: `notes/place-integration` (notes only,
  LLM-co-written — never upstream from it).
- ~/repos/niri is a plain checkout (no wtree layout): real code work goes
  on separate `feat/*` branches.

## Appendix: primary references

- place: `~/.config/niri/place/{layout,place,niri,arrange_gtk}.py`
- niri: see §3 table for file:line hooks
- Upstream: discussion #3793 (relative output placement, open), PR #4318
  (closed per LLM policy — no design verdict), PR #4618 (open-on-output
  fallbacks, merged 2026-09), issue #4702 (identical monitors), issue
  #4259 (output placement / cursor confinement edge cases)
- Docs: `docs/wiki/Development:-Design-Principles.md` (invisible state),
  `docs/wiki/Configuration:-Named-Workspaces.md` (open-on-output history),
  CONTRIBUTING.md (LLM policy)
