# INTEL — Mech Royale recon brief

Living brief for the combat bot. Scorer facts land after Hour 1 slot 1.

## Runtime

- Kit / local: Python 3.13, stdlib only, `python:3.13-slim` image.
- Scorer image: pending dump-platform (S1).
- Timing headroom: unknown until sleep probes. Protocol budget 50 ms, hello 20 s.
- Hard no's: none yet.

## Scorer facts

- S1 BB-C6CA6276 rejected; check.py ran in the match sandbox. stderr feedback is a **short tail** (upgrades JSON only). Put the payload you care about last and keep it small.
- Scorer `hello.config.upgrades` matches the kit recording (mortar/laser/lightning upgrade ids and muls). Treat kit config as live until proven otherwise.
- Local `check.py` hello.protocol `mechfight/1`, slot 3 in the recording.
- Map files: `v2/world.json`, `v2/terrain.i16`.
- Runtime OS/cgroup/python: pending compact S2 dump-runtime.
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

Unknown. Practice: 15 house bots, shared vision, no friendly fire, weakest-first.

## Movement

Unmeasured in matches. Protocol: slide on walls, step_up 1.2 m, jet for roofs, ~60 m climb on 100 energy.

## Combat

Unmeasured. First play probe: `aim-nearest-visible`.

## Economy

Unmeasured. Floor weapons 40, crates 36, NPCs 12, 3 supply drops.

## Algorithms

None promoted. Control after S2/S3: kit `py-starter` until a play probe beats it.

## Open questions

- Scorer OS / cgroup / `/arena` listing / Python in container (S1)
- stderr truncation length (S1)
- Whether scorer `hello.config` matches kit recording (S1)
- Remote `py-starter` score vs baseline 1.15
- `aim-nearest-visible` vs starter
- 50 ms cliff, house-bot hunt, roofs, loot
