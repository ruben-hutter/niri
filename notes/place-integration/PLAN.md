# Integrating `place` into niri — research & plan

**Status 2026-11-21: Part 1 shipped as PR1 (pending review). Parts 2–3
PIVOTED to an external Rust tool — they will not be proposed upstream.
place (the Python tool) was retired 2026-11-21: sources and state deleted,
no archive. §2–§4 below are now the surviving record of how it worked;
§6.7's lessons bind the Rust port.**

Written 2026-10-31 against upstream niri @ 2026-10-27 (line numbers in §3
are from that pass; re-verify against current main). Companion scratch
file: `/TASK.md` (branch root).

---

## 0. Where things stand (2026-11-21)

- **PR1** = `feat/output-roles` @ `4ab8aa07` on the fork
  (`ruben-hutter/niri`), single squashed commit on upstream/main
  `689c1e91`: `output { role N }` + role-anchored workspaces +
  `role_anchor` in `niri msg -j workspaces` / `WorkspacesChanged` +
  reload-transition logging + wiki section + tests (285 niri / 38
  niri-config / 3 niri-ipc green, fmt+clippy clean).
- ruben reviews the code and writes the PR description **himself**
  (LLM policy); source brief: `~/repos/niri/PR-DRAFT-output-roles.md`.
- **Pivot decision:** the arrangement engine, setup recognition,
  persistence, and wizard will NOT go upstream (learned/persisted state is
  almost certainly rejected). They become an **external Rust tool** (§6).
- place retired: `~/.config/niri/place/` and `~/.local/state/place/`
  deleted; the disabled autostart line removed; the `role N` lines in
  `conf/output.kdl` stay (supported by the PR build, which is now the
  system niri — see `~/.config/niri/ROLE-TEST-REVERT.md`).

## 1. Executive summary (updated 2026-11-21)

place kept niri workspaces arranged by *monitor role* across monitor-setup
changes. The substrate analysis (§3) proved the compositor had exactly one
missing *declarative* primitive — a configurable primary/role — and one
missing *learned* behavior — per-setup block distribution. Outcome:

- The declarative primitive went upstream as **PR1** (roles + anchoring).
  With it, workspaces already follow their monitor's role across setups.
- Everything learned/stateful ships **outside** the compositor: an
  external Rust tool talking to stock niri over IPC only. Same trick
  place.py proved works — but event-driven, typed, and with a real picker
  UI instead of title-hack polls and badge overlays.

## 2. What place was (record — code deleted 2026-11-21)

| file | LoC | role |
|---|---|---|
| `layout.py` | 300 | pure state machine: strip (`order`), active cut (`sizes`), per-setup cuts (`splits`), lifecycle verbs (`migrate/transition/arrive/settle/rekey/learn`), `plan()` → moves |
| `place.py` | 560 | daemon: 5 s poll, setup detect/register, converge loop, journalctl lock watch, wizard driver, align, pause/resume |
| `niri.py` | 60 | IPC seam (`niri msg` + Move/Focus raw actions) |
| `arrange_gtk.py` | 167 | badge overlay: click monitors to define desk order / roles |
| `test_layout.py` | 408 | unittest suite of the pure logic |

Model: workspaces form one canonical strip cut into blocks by monitor
*role* (importance order). A *setup* = monitor set + human-declared desk
order, hashed to an id. Each setup remembers its *split* (block sizes);
arriving re-cuts the strip so returning to a desk re-creates its
arrangement.

Key invariants worth keeping in any port:

- Same monitor set, different desk arrangement = different setup.
- Never learn during a transition (only while settled).
- Human-declared desk order beats niri's auto-layout guess.
- Monitor-*set* fallback match when arrangement hash differs (drift).

## 3. What niri has (verified 2026-10-27; re-check file:line on current main)

| place concept | niri counterpart |
|---|---|
| identity = make model serial | `OutputName` (`niri-config/src/output.rs:119`); workspaces store `original_outputs: Vec<OutputId>` normalized to make/model/serial (`src/layout/workspace.rs`) |
| setup change detection | events: `Layout::add_output/remove_output` (`src/layout/mod.rs:760,878`) via `Niri::add_output/remove_output` (`src/niri.rs:3078,3176`) |
| converge loop | direct synchronous calls: `move_workspace_to_output_by_id` (`src/layout/mod.rs:3482`), `Monitor::move_workspace_to_idx` (`src/layout/monitor.rs:1295`); externally reachable via the same IPC actions |
| **primary monitor** | `MonitorSet.primary_idx` — **was hardcoded 0**; PR1 makes it role-driven (lowest connected role, config-order tiebreak) and adds role-anchored workspaces. ← **the roles gap, now a PR** |
| focus restore | `last_active_workspace_id` per connector (`src/layout/mod.rs:358`); `focus-at-startup` per output exists (`niri-config/src/output.rs:71`) |
| lock watch | `Niri::is_locked()` internal; externally: check what the event stream exposes (verify during implementation) |
| align | `Niri::reposition_outputs()` (`src/niri.rs:2955`); IPC "set output position" exists (`niri-ipc/src/lib.rs:1090`) |
| wizard UI substrate | external tool brings its own (GTK4/iced/egui — §6 open question); in-compositor `src/ui/` only matters for PR1-free futures |
| persistence | **nothing** — niri writes no state files (this is why Part 3 can't go upstream) |
| **role anchor (new, PR1)** | `role_anchor` per workspace in `niri msg -j workspaces` + `WorkspacesChanged`; anchor moves logged at debug with reasons |

Built-in behavior the arrangement engine competes with: per-workspace
affinity (`find_preferred_output`, `src/layout/workspace.rs:479`), PR1's
role anchoring, named-workspace `open-on-output` pins with fallback lists
(PR #4618). Remaining gap = per-setup block distribution + persistence;
after reboot dynamic workspaces pile onto one monitor. Issue #4702
(identical monitors → workspaces on wrong monitor after power-cycle)
still demonstrates the identity ambiguity.

## 4. What the external tool must provide itself

place-in-compositor would have deleted the poll loop, converge-retry,
journalctl lock-watch, session token, pick-window title hack. Externally,
they come back in better form: the 5 s poll becomes an **event
subscription** (`niri msg event-stream` — verify the exact events cover
outputs, workspaces, and lock state); the lock watch becomes an event or
a state query; the pick-window hack disappears (the picker is a real GUI).
What remains to build: the pure state machine, the setup store, the
picker UI, align-by-declared-order.

## 5. THE SPLIT — outcome

Decided 2026-10-31, executed with the 2026-11-21 pivot:

- **Part 1 — roles in config (declarative)** → **PR1, done** (§0).
- **Part 2 — setup recognition + click-to-define** → external tool (§6).
  The wizard-as-config-editor principle carries over unchanged: clicking
  writes what the user would have written (role lines into the niri
  config; hot-reload applies them). Config stays the single source of
  truth for *importance*.
- **Part 3 — learned per-setup splits** → external tool (§6). Persistence
  moves to the tool's own state dir — no upstream consent needed;
  installing the tool *is* the opt-in. Cache-not-authority still applies:
  corrupt/stale file → ignore + rebuild; delete file → vanilla.
- The old "ask upstream the persistence question first" plan is **moot** —
  we're not asking. The argument kit is retired with it.

## 6. The external Rust tool — the idea

A small, niri-specific app users install alongside niri. It gives stock
niri the rest of the place workflow: **define a setup by clicking, keep
each setup's workspace arrangement, restore it on arrival.**

### What it does

1. **Setup definition (Part 2):** on an unknown monitor arrangement, the
   user opens the picker and clicks the monitors into desk order
   (left→right) and importance (role). Confirming writes the `role N`
   lines into the niri config (wizard-as-config-editor) and registers the
   setup (identity hash + desk order) in the tool's own store.
2. **Arrangement memory (Part 3):** the tool keeps each setup's block
   distribution and re-cuts the workspace strip on arrival; on departure
   everything collapses onto the surviving roles (niri's PR1 anchoring
   already does most of the per-workspace follow; the tool adds the
   *cuts* and the per-setup shape).
3. **Align (place `align`):** output positions follow the declared desk
   order via the existing "set output position" IPC.

### UI principle (ruben, 2026-11-21)

A simple centered picker dialog — the principle of the browser
screen-share chooser ("Choose what to share": tabs, clickable tiles,
explicit Share/Cancel): connected monitors shown as clickable tiles, two
click-passes (desk order, then importance), explicit confirm. Deliberately
NOT an overlay pinned onto the physical monitors (that was arrange_gtk's
badge approach). Open for the grilling session: static tiles vs live
thumbnails; drag vs click-passes; single dialog vs two steps; what shows
for identically-named monitors (#4702 ambiguity).

### Architecture sketch

- **crate `arrangement`** — pure state machine, dict-in/dict-out ported to
  typed structs: the strip/order/sizes/splits verbs from §2, with §6.7's
  binding principles (identity-based blocks, death propagation,
  last-known-role seeding, drift guard). Test suite ports 1:1.
- **crate `niri-ipc` glue** — `niri msg -j` typed via `niri-ipc` types
  where possible + the event socket; actions: move-workspace-to-output,
  focus, set-output-position.
- **binary** — daemon (event-driven; converges on setup change; paused
  while locked) + `setup` subcommand that opens the picker dialog.
- **state** — `~/.local/state/<name>/`: `state.json` (strip), `setups.json`
  (setup table), logs; atomic writes; delete = vanilla.
- **GUI toolkit** — TBD (grilling): place used GTK3 badges; candidates
  GTK4/libadwaita, iced, egui. Keep the dialog tiny and single-purpose.

### Roles dependency (design for both worlds)

With PR1 merged, roles are native: workspaces anchor themselves, the
tool's learned part shrinks (§6.7 point 5), and the tool only adds cuts
+ setup memory. If PR1 is rejected, the tool must still work on stock
niri: it keeps its own importance table instead of config roles and does
all anchoring itself via moves. **Design the state machine so the role
source is injectable** (config-provided vs tool-provided).

### Trigger surface (the still-open decision)

When does redistribution run? Hotplug only? Config reload? Resume from
suspend? Never mid-settle, never while locked. ruben asked for concrete
scenario examples before any code — that stands; present them in the
grilling session.

### Distribution

Single repo, MIT/Apache like niri, GitHub releases + crates.io. Uses only
niri IPC → version coupling limited to the IPC surface. Autostart is the
user's own `spawn-at-startup` line (place's pattern worked fine).

## 7. Design notes (carried from the in-compositor plan)

- Module shape: the pure crate above replaces the planned
  `src/layout/arrangement.rs`; the niri-side hooks table (§3) becomes the
  IPC/event surface; no config surface, no new niri IPC actions needed
  (PR1's `role_anchor` field is the only compositor-side addition).
- Restart semantics: place's way — persist id-blocks + derived counts;
  re-adopt the open strip on start; cut on first expanded arrival; no new
  identity scheme across sessions.
- Identical monitors (#4702): identities collide (niri "unnames"
  duplicates). Still open, now a *tool* decision: connector tiebreak,
  declared-order disambiguation, or documented ambiguity — pick in the
  grilling session.

### 6.7 Design amendment from the field (2026-11-03): identity-based memory

A real failure in place.py demonstrated the model weakness the Rust port
must not inherit. Stenogram: a setup's remembered split was count-based
(`[2,2,0]`); an incidental workspace had drifted it on 10-29; the
workspace later died at another setup, but the stored counts never learned
of the death; the stale cut then hijacked the next arrival and put a
workspace on the wrong monitor. Replay-proven against the pure module
(red on stored split, green on the correct one).

Binding principles for the arrangement crate:

1. **Arrangement memory references identity, never bare counts.** Per
   setup, store id-blocks (workspace ids per role monitor). Counts are a
   derived value kept only as the cross-session seed — workspace ids die
   with the session (`migrate` drops blocks, keeps counts, as place.py
   already did).
2. **Death propagation.** Closing a workspace must purge the id from
   every setup's blocks (and recompute that setup's derived counts).
   Counts alone cannot know which block a dead workspace belonged to —
   this was the root defect.
3. **Last-known-role seeding.** Maintain a per-workspace `last_role` map
   (niri-native analog: `last_active_workspace_id`). On arrival: use the
   setup's blocks where they exist; workspaces the setup has never seen
   get seeded by their last-known role.
4. **Drift guard.** Distrust any memory that references dead state; the
   cheap count-era proxy (reject a split whose sum > occupied workspaces)
   is worth carrying as an invariant/assertion even with id-blocks.
5. **Cross-session identity stays lossy** (no stable workspace id across
   reboot). If wanted later, a fingerprint bridge (match restored
   workspaces by sorted `app_id`s) is the natural extension — and PR1's
   declared roles shrink what must be learned in the first place: with
   roles declared, workspaces get an authoritative role target and the
   learned part of the model gets smaller.

Why this matters: the user-experienced model is **role affinity** ("this
app belongs on my second screen"), never strip-cut positions. Counts are
an approximation of that which breaks precisely on the transitions the
feature exists for.

(place.py disposition, for the record: state fixed manually; drift-guard
stopgap shipped there 2026-11-03, suite 52/52; no model rework — subsumed
here. Tool deleted 2026-11-21.)

## 8. Upstream process (now minimal)

- The only upstream ask is **PR1** — human-written text per the LLM
  policy (ruben reviews/rewrites everything; source brief exists).
- Never proposed upstream: persistence, wizard, learned state, arrangement
  engine.
- The external tool needs no upstream review at all: separate repo,
  talks over public IPC only. CONTRIBUTING's LLM policy governs niri
  contributions, not the tool repo — how ruben builds it is his call.
- Still worth watching: discussion #3793 (relative output placement —
  could obsolete the tool's align), issue #4702 (identity ambiguity).

## 9. Repo facts (2026-11-21)

- `~/repos/niri` is now a **wtree layout** (`.bare` + worktrees):
  `main/` = upstream mirror (local main @ upstream tip), `output-roles/` =
  PR1 branch, `notes/` = this branch. Feature work in dedicated worktrees.
- Fork `origin/main` reset to upstream (`689c1e91`); the full pre-squash
  series lives in `backup/pre-squash-output-roles` (local + origin).
- System niri = local PR build via `/usr/local/bin/niri` symlink →
  `output-roles/target/release/niri` (ops notes:
  `~/.config/niri/ROLE-TEST-REVERT.md`).
- place deleted 2026-11-21; the only surviving model documentation is this
  file and TASK.md.

## Appendix: references

- place sources: **deleted 2026-11-21, no archive** — §2/§6.7 above are
  the record of the design.
- PR draft brief: `~/repos/niri/PR-DRAFT-output-roles.md`.
- Upstream: discussion #3793 (relative output placement, open), PR #4318
  (closed per LLM policy — no design verdict), PR #4618 (open-on-output
  fallbacks, merged 2026-09), issue #4702 (identical monitors), issue
  #4259 (output placement / cursor confinement edge cases).
- Docs: `docs/wiki/Development:-Design-Principles.md` (invisible state),
  CONTRIBUTING.md (LLM policy — read before any upstream text).
