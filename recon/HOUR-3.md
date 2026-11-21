# Hour 3 — draft (confirm after Hour 2 meeting)

Do not execute this order if Hour 2 disagrees.

Steering inputs: idle **0.02**. Lead-gun **2.54** beats snap 2.42. sleep-35 **2.42** (35 ms legal). Still waiting: rust-aim-nv.

Candidates (pick exactly 4 at the meeting):

1. `break-los` or `hide-prism` — **promoted** after idle 0.02. `break-los` host PASS; `hide-prism` A* behind a prism (Hour-2 wait).
2. `kite` — deferred from Hour 2; commit vs strafe if rust/lead did not already reshape combat.
3. `walk-center` or `boost-center` — isolate boost noise if idle TTK is short (sound range boost 90 vs step 40).
4. `take-laser` or `silent-walk` — only if pathing/hunt is not the blocker.
5. If Rust ≈ 2.42: next compiled multiplex is **Rust 4 m A\* + aim-nv** (pathing + combat), not a third language.
6. Python `path-zone` — 4 m A\* to zone when nobody is visible, else aim-nv. 1-CPU Docker PASS (ready 501 ms, tick p50 1.0 max 1.8). Kill if score ≤ walk-to-zone.

Local already: 4 m A\* max 2.8 ms on 1 CPU — legal. 2 m A\* max 25 ms — Rust-only until proven. `break-los` and `path-zone` are implemented; do not submit them until the Hour 2 meeting.
