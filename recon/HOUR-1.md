# Hour 1 — precommitted slots

Steering: no prior meeting. Spend all 4.

1. `dump-platform` — platform + `hello.config` via fail-check stderr (config last on stderr).
2. dump remainder if S1 truncated; else `py-starter`.
3. `py-starter` or `ts-starter` (whichever S2 did not already run).
4. `aim-nearest-visible` — first combat intel.

Review after slot 4 before Hour 2.

## Hour 1 review (after S4 2.42)

1. **Learned**
   - S1: fail-check stderr is a short tail; bulky dumps die.
   - S2: compact ~2 KiB dump is allowed; Debian 13 / Python 3.13.18 / uid 65534 / 1 CPU / 512 MB; `/arena` has `complete` and `v2`.
   - S3: remote `py-starter` scores exactly **1.15** (baseline). Packaging is correct. UI is a single mean.
   - S4: `aim-nearest-visible` scores **2.42**. Chase+snap gun is live combat intel, not noise.

2. **Falsified**
   - Kit starter as combat control (half the score of chase-gun).
   - Large stderr dumps. Do not dump again unless `/arena/complete` is the top unknown (it is not).

3. **Promoted**
   - Controller: `aim-nearest-visible` (chase nearest visible mech, gun only, walk zone if nobody in sight).
   - Compact fail-check dumps for platform questions.
   - Python 3.13 stdlib-only image.

4. **Highest-value unknown**
   - House-bot hunt / time-to-kill on a sitting target (`idle`).
   - Lead vs snap on the same chase.
   - 50 ms cliff (`sleep-35` on the new control).
   - Commit vs kite (aim-nv is commit).

5. **Hour 2 slots** (replace the draft calendar)
   1. `idle`
   2. `lead-gun`
   3. `sleep-35` wrapping aim-nv
   4. `kite`

6. **Risk**
   - No dump. No ts-starter. No grinding 2.42. Next quota 2:04 AM EST.
