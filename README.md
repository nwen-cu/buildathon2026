# buildathon2026 — Mech Royale Hour-1 probe

MIT License

Team **BOAT-badass_of_all_time**. Submission repo: [nwen-cu/buildathon2026](https://github.com/nwen-cu/buildathon2026).

Bot kit and protocol from the official Mech Royale buildathon kit (`mechfight-kit`); this repo keeps the extracted kit locally for `check.py` only (not committed).

## Layout

- `bot.py` — probe bot; behavior from one-line `PROBE` file (`dump-platform`, `py-starter`, `aim-nearest-visible`).
- `mechfight.py`, `example_bot.py` — kit wrapper and reference example.
- `Dockerfile` — image entrypoint runs `bot.py` unbuffered.

## Local check

```powershell
uv run --python 3.13 python mechfight-kit/check/check.py -- python -u bot.py
```