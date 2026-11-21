# Hour 2 — steered after Hour 1 review

Quota: four free at 2:04 AM EST (rolling hour from S1 1:04 AM).

Control: `aim-nearest-visible` (2.42). Do not dump.

1. `idle` (Python) — house-bot TTK / weakest-first. Isolated; do not confound with language.
2. `lead-gun` (Python) — same chase as control, gun-speed lead vs snap.
3. `sleep-35` (Python aim-nv) — multiplex: 50 ms cliff **and** whether the promoted controller still plays when the tick is expensive.
4. `rs-aim-nv` (Rust replica of control) — multiplex: compiled runtime legal under 10 min build **and** score vs Python 2.42 **and** `/arena` world.json load in hello (planner substrate).

Kite deferred to Hour 3 so this hour buys a language decision. If Rust build/check fails, swap slot 4 to Python `kite` rather than skip the slot.

If idle dies at floor (score ~0), Hour 3 prefers survival/break-LOS over loot. If lead-gun beats 2.42, later combat probes lead. If Rust matches or beats 2.42, combat-bot default becomes Rust.

## Hour 2 review (after S8 rust 2.42)

1. **Learned**
   - Idle **0.02**: house bots commit immediately on a sitting target.
   - Lead-gun **2.54** beats snap **2.42**. Keep lead on later combat probes.
   - sleep-35 **2.42**: 35 ms work is legal remotely; lockstep scoring is slow when ticks are slow.
   - Rust snap **2.42** = Python snap. Image legal, map load real, language score-neutral on greedy chase.

2. **Falsified**
   - Standing still. Never idle.
   - Need for a third language (TS) this recon.
   - Switching the A/B vehicle off Python: lead 2.54 is still Python; Rust matched snap, not lead.

3. **Promoted**
   - Control policy: lead-gun chase (2.54).
   - Runtime budget: 35 ms.
   - Rust as planner substrate (4 m A* local PASS). Python stays the isolated A/B vehicle.

4. **Highest-value unknown**
   - Survival: hide behind a prism on damaged vs keep committing (idle 0.02).
   - Kite vs commit.
   - Boost noise vs walk (sound 90 vs 40) given shared-vision hunt.
   - Remote 4 m A* (`path-zone`) now that Rust is legal.

5. **Hour 3 slots**
   1. `hide-prism` (Python)
   2. `kite` (Python)
   3. `boost-center` (Python)
   4. `path-zone` (Rust 4 m A* + snap)
