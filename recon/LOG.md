# Recon log

## Local dump-platform (pre-submit)

- Hypothesis: dump bot fails check after writing platform JSON, ruler, and hello.config (config last on stderr).
- Probe: `dump-platform` / uncommitted local
- Allowed: local check FAIL as designed; ready in 97 ms; 1 bad line (`INVALID PROTOCOL LINE`); exited at tick 0.
- Score: n/a
- Verdict: promote dump layout for S1
- Intel: kit `hello.config` recorded in INTEL.md Scorer facts

## Hour 1

_(S1–S4 blocks go here)_
