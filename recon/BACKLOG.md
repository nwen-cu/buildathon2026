# Experiment backlog

Meetings reorder and drop items. This is a menu, not a queue.

## Platform and budgets

- `dump-platform` — stderr JSON + ruler; fail check after dump
- `ts-starter` in combat
- `sleep-20` / `sleep-35` / `sleep-45` on current control — Hour 2 slot 3 is `sleep-35`
- numpy / load `world.json` in hello / `/tmp` / threads / memory toward 512 MB
- one compiled runtime if Python is tight — Hour 2 slot 4 is **Rust aim-nv replica** (multiplex language + control + map load). Kite deferred. ts-starter only if Rust cannot build.

## Movement and survival

- `idle` once (hunt latency), then never again — Hour 2 slot 1
- `walk-center` vs `boost-center`
- silent-walk under 5 m/s
- `jump-jet-roof`
- `break-LOS` on damaged
- `zone-hug` vs `zone-edge`
- wall-slide vs straight line

## Combat

- `aim-nearest-visible` snap — 2.42
- lead-gun vs snap — **promoted control, 2.54**
- lead-gun vs snap — Hour 2 slot 2 (done)
- kite vs commit — Hour 2 slot 4
- face-attacker
- sound-turn
- spray-blind through trees
- laser vs mortar vs lightning as first `take`
- hold-trigger vs charge-and-release

## Economy

- nearest weapon vs ignore
- crate crash
- cube chase
- supply-drop contest
- NPC farm vs ignore
- offer pick 0 vs sensors vs armor

## Planners

- greedy
- potential field
- grid A* (kill immediately if over budget)
- utility over {fight, loot, roof, drop, flee}
