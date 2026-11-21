# Recon log

## Local dump-platform (pre-submit)

- Hypothesis: dump bot fails check after writing platform JSON, ruler, and hello.config (config last on stderr).
- Probe: `dump-platform` / uncommitted local
- Allowed: local check FAIL as designed; ready in 97 ms; 1 bad line (`INVALID PROTOCOL LINE`); exited at tick 0.
- Score: n/a
- Verdict: promote dump layout for S1
- Intel: kit `hello.config` recorded in INTEL.md Scorer facts

## Hour 1

### S1 dump-platform — BB-C6CA6276

- Hypothesis: fail-check stderr returns platform JSON, ruler, and hello.config (config last).
- Probe: `dump-platform` / `b3d1e189cf0916c071fa47c63ab283f120379325`
- Allowed: built; check FAIL as designed (tick 0 not JSON). Queue wait 39 ms. 3 of 4 slots left; next free 2:04 AM EST.
- Score: n/a (rejected, best unchanged)
- Verdict: isolate — stderr is a short **tail**. We got the end of `hello.config.upgrades` only. Platform dump was dropped. Config tail matches kit recording.
- Intel: scorer hello.config ≈ kit; truncation ~last few KB; S2 must be a tiny runtime dump.

### S2 dump-runtime — BB-919F0E8A

- Hypothesis: a ~2 KiB runtime dump survives the stderr tail.
- Probe: `dump-runtime` / `de154e34f13ae9c81964d588fb16875f80966039`
- Allowed: built; check FAIL as designed. Whole JSON + ruler returned.
- Score: n/a
- Verdict: promote compact dumps; kill further large config dumps. Python 3.13.18 + Debian 13 + 1 CPU cgroup confirmed.
- Intel: Runtime section filled. `/arena/complete` is a new unknown.

### S3 py-starter — BB-00F46ED6

- Hypothesis: remote kit example scores ~1.15 like the published baseline.
- Probe: `py-starter` / `7a8e6b54ef3defe133a1706ac8214b43abcdc007`
- Allowed: built, check passed, scored. Wait 84 ms.
- Score: **1.15** (exactly baseline). Entered in the final as current best. Board rank ~158.
- Verdict: promote as control. Result UI is a single mean, no per-match places.
- Intel: scorer matches the published starter score; our packaging is correct.

### S4 aim-nearest-visible — BB-2E0308D5

- Hypothesis: snap-aim and chase the nearest visible mech, gun only, no loot/boost — beats idle walking-to-zone.
- Probe: `aim-nearest-visible` / `20a11d71ebb4419c319913e9d922ae23581b656e`
- Allowed: built, check passed, scored. Wait 125 ms.
- Score: **2.42** (vs py-starter 1.15). Entered in the final as current best. Board rank ~132.
- Verdict: **promote as control.** Fighting nearest visible mech more than doubles starter placement. Kit loot/jump/boost-out-of-zone is not the combat core.
- Intel: Combat section; Algorithms control = aim-nv chase+snap gun.

## Local (wait window before Hour 2)

- Language: kit `example.ts` via bun PASSes check (ready 179 ms, p50 0.2 ms). Python probes ready ~45 ms host / ~300–400 ms in 1-CPU Docker. **Stay on Python 3.13** for submits.
- All play PROBE modes PASS host check.py (0 late except sleep probes by design). `sleep-20` p50 20.5 ms; `sleep-35` 35.6 ms; `sleep-45` 45.6 ms.
- 1-CPU 512 MB Docker (match flags): idle PASS p50 0.9 ms; sleep-35 PASS 0 late p50 36.7 ms; sleep-45 PASS but **2/300 late**, max 91.9 ms.
- Recorded check stream (300 obs): visible mech **56%** of ticks, range p50 140 m / p90 150 m. Horizontal lead at gun 252 m/s: p50 4.4 m, p90 7.6 m. Stream events: 13 damaged, 10 hit. Sounds include gun (62), laser, explosion, crate, death, boost.
- `hello.config.mech` key is `walk_speed` (11.7), not `walk`. `world.json` 300 KB / 3.4 ms load.
- Rust `mechbot` (serde_json, rust:1.85-bookworm → debian slim) 1-CPU Docker: aim-nv/idle/lead-gun PASS. ready ~200–230 ms. tick p50 **0.9 ms** (same floor as Python idle in Docker). Hello `world.json` **map_loaded=true load_ms≈5–6 prisms=259**. Compile after deps ~2.5 s locally; first scorer build pays the rust image pull (fits 10 min).
- 1-CPU Docker grid/A* (Python, `/arena/v2/world.json`): 156 ground-blocking prisms. 4 m A* max **2.8 ms** (legal). 2 m A* max **25 ms** (tight). `break-los` host check PASS p50 0.1 ms. `path-zone` 1-CPU Docker PASS ready 501 ms, tick p50 1.0 max 1.8.
- Hour-2 wait: `kite` host PASS p50 0.1; **1-CPU Docker PASS** ready 298 ms, tick p50 **0.9** max **1.5**. `silent-walk` and `boost-center` host PASS p50 0.1.
- `hide-prism` (lead chase; on damaged, 4 m A* to a cell with a prism between self and attacker). Host PASS ready 243 ms p50 0.1 max 1.1. **1-CPU Docker PASS** ready 512 ms, tick p50 **1.0** max **1.9**, 1 queued drop (still 0 late). Hour-3 survival candidate ahead of naive `break-los`.
- Rust `h2-rs-aim-nv` 1-CPU Docker recheck before slot 4: PASS ready 173 ms, tick p50 **0.9** max **1.6**, `map_loaded=true load_ms=5.98 prisms=259`.

## Hour 2

Quota opens 2:04 AM EST. Control: aim-nv 2.42.

### S5 idle

- Hypothesis: a motionless mech is hunted quickly (weakest-first), so idle scores near the floor and gives TTK/hunt intel.
- Probe: `idle` / `01526ab20ebc80ac464f3ed684adb8af0c568dcf`
- Allowed: built, check passed, scored. Wait 166 ms.
- Score: **0.02**
- Verdict: **kill idle forever.** House bots hunt a motionless mech to near-first-dead. Survival/break-LOS is now a top Hour-3 unknown. Do not confound with language.
- Intel: House bots — TTK on idle is match-start; weakest-first is real.

### S6 lead-gun

- Hypothesis: same chase as aim-nv 2.42, aim at gun-speed lead instead of snap; recorded vis range p50 140 m implies ~4.4 m lead.
- Probe: `lead-gun` / `aaf59897a1fee7b6659fe4519e73fc9570b9e216`
- Allowed: built, check passed, scored. BB-BD3210F4 Sat 2:07 AM. Wait 116 ms.
- Score: **2.54** (vs snap aim-nv 2.42). Entered in the final as current best. Board rank ~138.
- Verdict: **promote lead.** +0.12 mean from gun-speed lead at vis p50 140 m. Later combat probes lead, not snap.
- Intel: Combat — lead is the new control.

### S7 sleep-35

- Hypothesis: wrapping aim-nv in 35 ms sleep still scores near 2.42 remotely; isolates the 50 ms cliff without changing policy.
- Probe: `sleep-35` / `084f3405104d89470383150fd743a5137940c953`
- Allowed: built, check passed, scored. BB-8CE1E77C Sat 2:10 AM. Wait 481 ms. Fetching/scoring took ~15 min (slow ticks).
- Score: **2.42** (exactly aim-nv snap). Best unchanged at lead-gun 2.54.
- Verdict: **promote 35 ms as a legal remote budget.** Policy still plays; mean matches snap 2.42. Do not spend another slot on sleep-45. Combat work under 35 ms is not leaving points on the table vs greedy.
- Intel: Runtime — 35 ms is the remote-legal work window.

### S8 rust-aim-nv

- Hypothesis: Rust replica of snap aim-nv builds under 10 min, scores ≈ Python 2.42, and hello-loads world.json (259 prisms).
- Probe: `aim-nearest-visible` (Rust image) / _(SHA at submit)_
- Allowed: _(pending)_
- Score: _(pending)_
- Verdict: _(pending)_
- Intel: Language + algorithms substrate
