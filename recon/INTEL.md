# INTEL — Mech Royale recon brief

Living brief for the combat bot. Scorer facts land after Hour 1 slot 1.

## Runtime

- Image: `python:3.13-slim` is legal. Scorer Python **3.13.18** (GCC 14.2.0) at `/usr/local/bin/python3`.
- OS: Debian GNU/Linux 13 (trixie) container on **GCP Ubuntu 24.04** kernel `7.0.0-1017-gcp`, x86_64, hostname `0aa17f502e6a`.
- uid/gid **65534**. cgroup `cpu.max` = `100000 100000` (1 CPU). cgroup `memory.max` = 536870912 (512 MB).
- `os.cpu_count()` reports **32** and `/proc/meminfo` shows the **host** (~126 GB). Do not size thread pools from nproc/meminfo; obey cgroup.
- Import json/math ~0.004 ms. Local 1-CPU Docker: idle p50 **0.9 ms** (host 0.1 ms). `sleep-35` p50 **36.7 ms**, 0 late. `sleep-45` p50 46.6 ms, **2/300 late** (still <2% so check PASS) max 91.9 ms. Combat work should stay **under ~35 ms**; 45 ms is the local cliff.
- `world.json` (~300 KB) loads in **3.4 ms** locally; fine in hello, not every tick.
- Hard no's: none yet. Stdlib-only Python is allowed. Do not size thread pools from host nproc/meminfo.
- Language: **Python 3.13 is the A/B vehicle.** Rust snap aim-nv scored **2.42** (BB-EADD5BC0) — identical to Python snap. Greedy JSON ticks dominate; language is score-neutral on chase-gun. **Rust is legal** (built, mapped 259 prisms, wait 31 ms) and is the planner runtime for 2 m / terrain / LOS. 4 m A* is already legal in Python (max 2.8 ms) and now also in Rust (host/1-CPU hide-prism p50 0.9 ms). Skip `ts-starter`. Next compiled multiplex is Rust `path-zone` (4 m A* + snap), not a third language.

## Scorer facts

- S1 BB-C6CA6276 rejected; check.py ran in the match sandbox. stderr feedback is a **short tail** (upgrades JSON only). Put the payload you care about last and keep it small.
- Scorer `hello.config.upgrades` matches the kit recording (mortar/laser/lightning upgrade ids and muls). Treat kit config as live until proven otherwise.
- Local `check.py` hello.protocol `mechfight/1`, slot 3 in the recording.
- Map files: `v2/world.json`, `v2/terrain.i16`.
- S2 BB-919F0E8A compact dump **fit entirely** in the stderr tail (ruler A000–A031 present). Keep dumps under a couple of KB.
- `/bot` is the repo copy. `/tmp` empty at hello. `/arena` has **`complete` and `v2`** (kit only ships `v2`). `hello.map.files` still `v2/world.json`, `v2/terrain.i16`.
- hello.slot in check recording is 3; protocol `mechfight/1`.
- Recorded `hello.config` (kit stream; treat as live unless S1 differs):

Match: tick_hz 20, decide_every 2, duration_s 480, 16 mechs.

Bot: tick_budget_ms 50, hello_budget_ms 20000, coast_s 1, offer_timeout_s 5, max_line_bytes 65536.

Mech: r 2.5 h 9 eye 8 muzzle 7 hp 1400 walk 11.7 boost 19.5 g-accel 58.5 air-accel 12 gravity 22 jump 11 jet 34 max_rise 10 step_up 1.2 max_slope 50 energy 100 regen 18 delay 0.6 boost_drain 17 jet_drain 15 turn 4 pitch 3 min_pitch -0.7 max_pitch 1.3 regen_delay 5 regen_per_s 45 charge_move 0.5 self_damage 0.5 armor 0 rear_mult 1.5 front_arc 180.

Vision: range 200 fov 120 near 45 canopy_blocks true show_enemy_hp true memory_s 5.

Sound ranges: step 40, boost 90, gun 320, mortar/explosion 420, laser 380, laser_charge 140, lightning 160, jet 90, death/drop 600, crate 220, npc_gun 200, reload 60. step_speed 5, step_every 0.6, bearing_noise_deg 7.

Weapons: gun 45/0.24s/252 m/s/230 m/mag 12/reload 2.4; mortar 200 splash 160 r7 cooldown 2.6 speed 72; laser 315 charge 0.75 min 0.15 hold 2 cd 1.65 range 280 cookoff 0.6; lightning 60/0.25s/45 m cone 30 chain 2 energy 3 short 2 s.

Cubes: pickup 3.5, +4% hp/dmg cap 10, size +5%/cube, death extra 2, thresholds 1,3,6,10,15,21,28,36,45.

Crates 36, 25% roof, hp 300, 1 cube. Loot floor 40, crate weapon 0.5, weights mortar/laser/lightning 1, pickup_radius 4, equip_s 1.

NPC 12, hp 350, 2 cubes. Drops at 70/140/210 s, fall 25, r 8, capture 4 s, 3 cubes.

Zone start (41.3, -72.8) r 560; phases r 380/240/140/70/25/0 dps 20/35/60/100/160/260.

## House bots

Published: 15 house bots, shared vision, no friendly fire, weakest-first.
- **Idle scores 0.02** (BB-1B56E18D). They commit immediately on a sitting target; almost always first-dead across 48 matches.
- Aim-nv **2.42** / lead-gun **2.54** vs idle **0.02**: moving + shooting is the difference between place ~13 and place 16. Never stand still. Board **145 / 180** at 2.54 (Sat 2:33 AM). Top practice ~5.60.
- They can be damaged (lead-gun is not feeding 0.02). Hunt is real; they are not spectators.

## Movement

Aim-nv walks full speed toward the target (or zone) with **no boost** and still scored 2.42, so walking the Oval is enough to get guns on house bots. Boost/jet/roofs unmeasured. Protocol: slide on walls, step_up 1.2 m, jet for roofs, ~60 m climb on 100 energy.

## Combat

- **Promoted:** chase nearest visible `kind==mech`, **gun-speed lead** (252 m/s), fire gun only (`fire[True, False]`). Score **2.54** (BB-BD3210F4).
- Snap-aim same chase is **2.42**. Lead is worth +0.12; keep it.
- Kit starter (loot + jump + boost-out-of-zone + fire both) is **1.15**. Do not inherit loot/jump as the combat core.
- Special weapons unmeasured.

## Economy

Unmeasured in isolation. Starter loots and scores worse than a no-loot chaser, so floor weapons are **not** required for a 2.x practice score. Crates/cubes/NPCs/drops still unknown as contests.

## Algorithms

**Control = greedy chase nearest visible mech + gun-speed lead.** Legal under budget. Lead 2.54 > snap 2.42 > starter 1.15.

Local map (kit `v2/world.json`): bounds ~1067×722 m, **259 prisms** (223 building / 27 stadium / 9 bridge), **156 ground blockers** (base < 2.5 m and top > 1.2 m). Terrain heightfield 1070×724 @ 1 m.

1-CPU 512 MB Docker A* on building occupancy (not terrain):
- **4 m grid** (48k cells, 48 KB): hello raster **122 ms**, A* **0.04–2.8 ms**. Legal in Python under 50 ms.
- **2 m grid** (194k cells, 189 KB): raster **422 ms**, A* **0.07–25 ms**. Fits locally; too little headroom next to JSON (~0.9 ms) and aim if we also sleep or do LOS. Rust is for 2 m / terrain / LOS, not for 4 m greedy pathing.

Do not kill Python for a 4 m planner. Do not run 2 m A* on Python until a 1-CPU probe says late% is 0.

Local cover_step on the Oval (400 random open cells, threat 80 m): ring 10 (40 m) hits **48%**; ring 15 hits 61%; ring 20 hits 73%. Misses fall back to flee+boost. Hour 3 `hide-prism` uses ring 10. If it ties lead, Hour 4 bumps ring before blaming the hide idea.

## Open questions

- House-bot TTK on idle: **answered** — 0.02, hunt is immediate
- Lead vs snap: **answered** — lead **2.54** beats snap **2.42** (BB-BD3210F4). Use lead.
- Remote 50 ms cliff: **answered** — `sleep-35` scores **2.42** (BB-8CE1E77C), same as snap aim-nv. 35 ms work is legal; scoring ran slow (~15 min) because lockstep waits. Keep combat under 35 ms. Do not probe sleep-45.
- Kite vs commit; hide-prism on damaged (local check PASS; Hour 3 slot 1)
- Remote 4 m A* in a match: local PASS (Python hide-prism / path-zone; Rust hide-prism / path-zone). Hour 3 slot 4 is Rust `path-zone`.
- Boost noise / silent-walk / roofs
- `/arena/complete` vs `v2` (deferred; combat unknowns rank higher)
- nproc-vs-cgroup thread safety (deferred; no threads in control)
