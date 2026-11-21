# Hour 3 — steered after Hour 2 review

Quota: four free at 3:05 AM EST (rolling hour; UI: next slot 3:05 AM).

Control: `lead-gun` (2.54). Rust snap = Python snap = 2.42. Do not dump. Do not idle.

1. `hide-prism` (Python) — on damaged, 4 m A* to a cell with a prism between self and attacker; else lead-gun. Survival vs commit after idle 0.02.
2. `kite` (Python) — strafe-away vs commit. Deferred from Hour 2.
3. `boost-center` (Python) — isolate boost noise/speed (sound 90 vs step 40) while still fighting when visible.
4. `path-zone` (Rust) — compiled 4 m A* to zone when nobody is visible, else snap chase. Multiplex: remote planner + language now that Rust is legal.

If hide-prism local check regresses before submit, swap slot 1 to `break-los`. If Rust `path-zone` cannot be armed in time, swap slot 4 to Python `path-zone` rather than skip.

Later combat probes still lead, not snap. Do not spend a slot on rust-lead until hide/kite/boost land.
