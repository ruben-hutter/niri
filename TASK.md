# TASK — niri place integration → PR1 + external tool

Scratch context for this branch (`notes/place-integration`). Full plan:
[`notes/place-integration/PLAN.md`](notes/place-integration/PLAN.md) — §6 is
the external-tool idea, §6.7 the binding state-machine lessons.

> **Branch warning:** research notes only, co-written with an LLM. niri's
> CONTRIBUTING.md forbids LLM-written upstream content — never open PRs,
> issues, or comments from this branch, and write all upstream-facing text
> yourself.

## Status (2026-11-21)

- **PR1 done and pushed**: `feat/output-roles` @ `4ab8aa07` (single commit:
  roles + role-anchored workspaces + `role_anchor` IPC + reload logging +
  wiki + tests), rebased onto upstream/main `689c1e91`, on the fork.
  Gates green (285/38/3, fmt, clippy-clean-except-upstream).
- **Next: ruben reviews the code, writes the PR description himself**
  (source brief: `~/repos/niri/PR-DRAFT-output-roles.md`), opens the PR
  `ruben-hutter:feat/output-roles` → `niri-wm:main`.
- **Strategy pivot (2026-11-21):** Parts 2–3 are NOT proposed upstream.
  They become an **external Rust GUI tool** (PLAN §6): picker dialog
  (screen-share-chooser principle) for desk order + roles; strip-cut
  arrangement engine per §6.7; own state dir; niri IPC only. Grilling
  session before any code.
- **place retired 2026-11-21**: `~/.config/niri/place/` and
  `~/.local/state/place/` deleted (no archive); autostart line removed;
  `role N` lines stay in `conf/output.kdl`; system niri = PR build via
  `/usr/local/bin/niri` (ops: `~/.config/niri/ROLE-TEST-REVERT.md`).
  PLAN §2/§6.7 are the surviving design record.

## Decisions log (compressed; details in PLAN.md)

1. **2026-10-31 — split into parts**: Part 1 roles (declarative,
   upstreamable), Part 2 click-to-define, Part 3 learned splits
   (opt-in persistence).
2. **2026-11-03 — identity-based memory amendment** (§6.7): the Zen
   incident proved count-based splits hijack arrivals; id-blocks +
   death propagation + last-known-role seeding are binding.
3. **2026-11-06 — strategy pivot #1**: no Matrix post; build Part 1,
   submit the rest separately. PR1 implemented on `feat/output-roles`.
4. **2026-11-17 — squash** into one commit; reload-transition logging
   added to the observability story.
5. **2026-11-21 — rebase + ship + pivot #2**: rebased onto fresh
   upstream (clean), logging folded in, single-commit branch pushed;
   Parts 2–3 → external Rust tool; place retired. Upstream gets only
   PR1.

## Next steps

1. ruben: review `feat/output-roles`, write PR text, open the PR.
2. Grilling session: external tool (PLAN §6 open questions — name,
   toolkit, trigger-surface scenarios first, roles-dependency,
   #4702 ambiguity handling).
