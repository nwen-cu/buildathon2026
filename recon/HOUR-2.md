# Hour 2 — steered after Hour 1 review

Quota: four free at 2:04 AM EST (rolling hour from S1 1:04 AM).

Control: `aim-nearest-visible` (2.42). Do not dump.

1. `idle` (Python) — house-bot TTK / weakest-first. Isolated; do not confound with language.
2. `lead-gun` (Python) — same chase as control, gun-speed lead vs snap.
3. `sleep-35` (Python aim-nv) — multiplex: 50 ms cliff **and** whether the promoted controller still plays when the tick is expensive.
4. `rs-aim-nv` (Rust replica of control) — multiplex: compiled runtime legal under 10 min build **and** score vs Python 2.42 **and** `/arena` world.json load in hello (planner substrate).

Kite deferred to Hour 3 so this hour buys a language decision. If Rust build/check fails, swap slot 4 to Python `kite` rather than skip the slot.

If idle dies at floor (score ~0), Hour 3 prefers survival/break-LOS over loot. If lead-gun beats 2.42, later combat probes lead. If Rust matches or beats 2.42, combat-bot default becomes Rust.
