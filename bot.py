"""Hour-1 recon probe: mode from PROBE file (one line)."""
import json
import math
import os
import platform
import sys
import time
import uuid

import mechfight as mf

_PROBE_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "PROBE")
_SECRET_RE = (
    "TOKEN",
    "SECRET",
    "KEY",
    "PASSWORD",
    "CREDENTIAL",
)


def _read_probe() -> str:
    env = os.environ.get("PROBE", "").strip()
    if env:
        return env
    try:
        with open(_PROBE_PATH, encoding="utf-8") as f:
            line = f.readline().strip()
            return line or "dump-platform"
    except OSError:
        return "dump-platform"


def _secret_key(name: str) -> bool:
    upper = name.upper()
    return any(part in upper for part in _SECRET_RE)


def _dir_listing(path: str) -> list[str] | str:
    try:
        return sorted(os.listdir(path))
    except OSError as e:
        return f"err:{e}"


def _meminfo_snippet() -> dict[str, str] | str:
    try:
        with open("/proc/meminfo", encoding="utf-8") as f:
            lines = [ln.strip() for ln in f.readlines()[:8]]
        out: dict[str, str] = {}
        for ln in lines:
            if ":" in ln:
                k, v = ln.split(":", 1)
                out[k.strip()] = v.strip()
        return out
    except OSError:
        return "absent"


def _cgroup_hints() -> dict[str, str]:
    hints: dict[str, str] = {}
    for rel in (
        "/sys/fs/cgroup/memory.max",
        "/sys/fs/cgroup/memory/memory.limit_in_bytes",
        "/sys/fs/cgroup/cpu.max",
        "/sys/fs/cgroup/cpu/cpu.cfs_quota_us",
    ):
        try:
            with open(rel, encoding="utf-8") as f:
                hints[rel] = f.read().strip()[:120]
        except OSError:
            pass
    return hints


def _os_release() -> dict[str, str] | str:
    try:
        data: dict[str, str] = {}
        with open("/etc/os-release", encoding="utf-8") as f:
            for ln in f:
                ln = ln.strip()
                if ln and "=" in ln and not ln.startswith("#"):
                    k, v = ln.split("=", 1)
                    data[k] = v.strip().strip('"')
        return data
    except OSError:
        return "absent"


def _uid_gid() -> dict[str, int | str]:
    out: dict[str, int | str] = {}
    for label, getter in (("uid", getattr(os, "getuid", None)), ("gid", getattr(os, "getgid", None))):
        if getter is not None:
            try:
                out[label] = getter()
            except OSError as e:
                out[label] = str(e)
        else:
            out[label] = "n/a"
    return out


def _import_timing() -> dict[str, float]:
    timings: dict[str, float] = {}

    def measure(name: str, fn) -> None:
        t0 = time.perf_counter()
        fn()
        timings[name] = round((time.perf_counter() - t0) * 1000, 3)

    measure("json", lambda: __import__("json"))
    measure("math", lambda: __import__("math"))
    return timings


def _env_report() -> dict[str, object]:
    keys = sorted(os.environ)
    values: dict[str, str] = {}
    for k in keys:
        if not _secret_key(k):
            values[k] = os.environ[k]
    return {"keys": keys, "values": values}


def _stderr_ruler(segments: int = 256) -> str:
    return "".join(f"A{i:03d}" for i in range(segments))


def _dump_platform(hello: mf.Hello) -> None:
    # Scorer returns the *end* of stderr on check failure. Write bulky
    # platform first, ruler, then hello.config last so a tail still has it.
    probe_id = str(uuid.uuid4())
    report = {
        "probe": "dump-platform",
        "probe_id": probe_id,
        "uname": platform.uname()._asdict(),
        "os_release": _os_release(),
        "uid_gid": _uid_gid(),
        "nproc": os.cpu_count(),
        "meminfo": _meminfo_snippet(),
        "cgroup": _cgroup_hints(),
        "python": sys.version,
        "executable": sys.executable,
        "platform": sys.platform,
        "env": _env_report(),
        "ls": {
            "/": _dir_listing("/"),
            "/bot": _dir_listing("/bot"),
            "/arena": _dir_listing("/arena"),
            "/tmp": _dir_listing("/tmp"),
        },
        "hello_protocol": hello.get("protocol"),
        "hello_slot": hello.get("slot"),
        "hello_map": hello.get("map"),
        "import_ms": _import_timing(),
    }
    sys.stderr.write(json.dumps(report, separators=(",", ":"), default=str) + "\n")
    sys.stderr.write("RULER_START " + _stderr_ruler() + " RULER_END\n")
    sys.stderr.write(
        json.dumps(
            {"probe_id": probe_id, "hello_config": hello.get("config")},
            separators=(",", ":"),
            default=str,
        )
        + "\n"
    )
    sys.stderr.flush()


class DumpBot:
    name = "h1s1-dump-platform"

    def __init__(self, hello: mf.Hello) -> None:
        _dump_platform(hello)

    def act(self, obs: mf.Observation) -> mf.Action:
        sys.stdout.write("INVALID PROTOCOL LINE\n")
        sys.stdout.flush()
        sys.exit(1)


def _dump_runtime(hello: mf.Hello) -> None:
    # Keep this under ~2 KiB so the scorer's stderr tail includes all of it.
    report = {
        "probe": "dump-runtime",
        "ls": {
            "/": _dir_listing("/"),
            "/bot": _dir_listing("/bot"),
            "/arena": _dir_listing("/arena"),
            "/tmp": _dir_listing("/tmp"),
        },
        "hello_protocol": hello.get("protocol"),
        "hello_slot": hello.get("slot"),
        "hello_map": hello.get("map"),
        "nproc": os.cpu_count(),
        "meminfo": _meminfo_snippet(),
        "cgroup": _cgroup_hints(),
        "os_release": _os_release(),
        "uid_gid": _uid_gid(),
        "uname": platform.uname()._asdict(),
        "python": sys.version,
        "executable": sys.executable,
        "platform": sys.platform,
        "import_ms": _import_timing(),
        "ruler": _stderr_ruler(32),
    }
    sys.stderr.write(json.dumps(report, separators=(",", ":"), default=str) + "\n")
    sys.stderr.flush()


class DumpRuntimeBot:
    name = "h1s2-dump-runtime"

    def __init__(self, hello: mf.Hello) -> None:
        _dump_runtime(hello)

    def act(self, obs: mf.Observation) -> mf.Action:
        sys.stdout.write("INVALID PROTOCOL LINE\n")
        sys.stdout.flush()
        sys.exit(1)


class PyStarterBot:
    name = "h1s2-py-starter"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        x, y, z = me["pos"]
        goal = obs["zone"]["next_center"]
        act: mf.Action = {"tick": obs["tick"]}
        if obs["offer"]:
            act["pick"] = 0
        near = [p for p in obs["pickups"] if math.dist(p["pos"][:2], (x, y)) < self.reach]
        if me["weapons"][1] is None and near and near[0]["kind"] != "gun":
            act["take"] = near[0]["id"]
        enemies = sorted(
            (m for m in obs["mechs"] if m["kind"] == "mech"),
            key=lambda m: math.dist(m["pos"], me["pos"]),
        )
        yaw = math.atan2(goal[1] - y, goal[0] - x)
        act.update(
            move=[math.cos(yaw), math.sin(yaw)],
            aim=[yaw, 0.0],
            boost=not me["in_zone"],
            jump=obs["tick"] % 40 == 0,
        )
        if enemies:
            t = enemies[0]["pos"]
            aim = [
                math.atan2(t[1] - y, t[0] - x),
                math.atan2(t[2] - z - 2.5, math.dist(t[:2], (x, y))),
            ]
            act.update(aim=aim, fire=[True, True])
            mf.draw.line([x, y, z + 7], [t[0], t[1], t[2] + 4.5], "#ff5040")
        mf.draw.point([goal[0], goal[1], z], "#40c0ff", 6)
        mf.draw.label(f"hp {me['hp']:.0f} · {len(enemies)} in sight")
        return act


def _xy(pos) -> tuple[float, float]:
    return (float(pos[0]), float(pos[1]))


def _toward(src, dst) -> list[float]:
    dx = dst[0] - src[0]
    dy = dst[1] - src[1]
    n = math.hypot(dx, dy)
    if n < 1e-6:
        return [0.0, 0.0]
    return [dx / n, dy / n]


def _aim_at(me_pos, target_pos, eye: float = 2.5) -> list[float]:
    x, y, z = me_pos
    tx, ty, tz = target_pos
    yaw = math.atan2(ty - y, tx - x)
    horiz = math.dist((tx, ty), (x, y))
    pitch = math.atan2(tz - z - eye, horiz)
    return [yaw, pitch]


def _zone_goal(obs: mf.Observation):
    zone = obs.get("zone") or {}
    return zone.get("next_center") or zone.get("center")


def _visible_mechs(obs: mf.Observation):
    return [m for m in obs["mechs"] if m.get("kind") == "mech"]


def _nearest_mech(obs: mf.Observation, me):
    vis = _visible_mechs(obs)
    if not vis:
        return None
    return min(vis, key=lambda m: math.dist(m["pos"], me["pos"]))


def _gun_speed(me) -> float:
    weapons = me.get("weapons") or []
    if weapons:
        w0 = weapons[0]
        if isinstance(w0, dict):
            stats = w0.get("stats") or {}
            try:
                return float(stats.get("speed", 252.0))
            except (TypeError, ValueError):
                pass
    return 252.0


def _lead_pos(me, target) -> list[float]:
    speed = max(_gun_speed(me), 1.0)
    dist = math.dist(target["pos"], me["pos"])
    t = dist / speed
    vel = target.get("vel") or [0.0, 0.0, 0.0]
    return [target["pos"][i] + float(vel[i]) * t for i in range(3)]


def _offer_and_take(act: mf.Action, obs: mf.Observation, me, reach: float, prefer: str | None = None) -> None:
    if obs.get("offer"):
        act["pick"] = 0
    near = [p for p in obs.get("pickups") or [] if math.dist(_xy(p["pos"]), _xy(me["pos"])) < reach]
    if not near:
        return
    weapons = me.get("weapons") or [None, None]
    slot1 = weapons[1] if len(weapons) > 1 else None
    if slot1 is not None:
        return
    chosen = None
    if prefer:
        for p in near:
            if p.get("kind") == prefer:
                chosen = p
                break
    if chosen is None:
        chosen = near[0]
        if chosen.get("kind") == "gun" and prefer is None:
            return
    act["take"] = chosen["id"]


def _walk_zone(act: mf.Action, me, obs: mf.Observation, boost: bool | None = None, scale: float = 1.0) -> None:
    goal = _zone_goal(obs)
    x, y, z = me["pos"]
    if goal is None:
        act.update(move=[0.0, 0.0], aim=me.get("aim", [0.0, 0.0]))
        return
    move = _toward((x, y), goal)
    act.update(
        move=[move[0] * scale, move[1] * scale],
        aim=[math.atan2(goal[1] - y, goal[0] - x), 0.0],
    )
    if boost is None:
        act["boost"] = not me.get("in_zone", True)
    else:
        act["boost"] = boost


class AimNearestVisibleBot:
    name = "h1s4-aim-nv"

    def __init__(self, hello: mf.Hello) -> None:
        self.hello = hello

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        x, y, z = me["pos"]
        act: mf.Action = {"tick": obs["tick"]}
        target = _nearest_mech(obs, me)
        if target is not None:
            yaw, pitch = _aim_at(me["pos"], target["pos"])
            act.update(
                move=[math.cos(yaw), math.sin(yaw)],
                aim=[yaw, pitch],
                fire=[True, False],
            )
        else:
            _walk_zone(act, me, obs, boost=False)
        return act


class IdleBot:
    name = "h2-idle"

    def __init__(self, hello: mf.Hello) -> None:
        self.hello = hello

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        return {
            "tick": obs["tick"],
            "move": [0.0, 0.0],
            "aim": me.get("aim", [0.0, 0.0]),
            "boost": False,
            "jump": False,
        }


class LeadGunBot:
    name = "h2-lead-gun"

    def __init__(self, hello: mf.Hello) -> None:
        self.hello = hello

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        target = _nearest_mech(obs, me)
        if target is not None:
            lead = _lead_pos(me, target)
            yaw, pitch = _aim_at(me["pos"], lead)
            act.update(
                move=[math.cos(yaw), math.sin(yaw)],
                aim=[yaw, pitch],
                fire=[True, False],
            )
        else:
            _walk_zone(act, me, obs, boost=False)
        return act


class BoostCenterBot:
    name = "h2-boost-center"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        _walk_zone(act, me, obs, boost=True)
        target = _nearest_mech(obs, me)
        if target is not None:
            act.update(aim=_aim_at(me["pos"], target["pos"]), fire=[True, True])
        return act


class WalkCenterBot:
    name = "h2-walk-center"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        _walk_zone(act, me, obs, boost=False)
        target = _nearest_mech(obs, me)
        if target is not None:
            act.update(aim=_aim_at(me["pos"], target["pos"]), fire=[True, True])
        return act


class SilentWalkBot:
    name = "h2-silent-walk"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]
        mech = (hello.get("config") or {}).get("mech") or {}
        walk = float(mech.get("walk_speed") or 11.7)
        self.scale = min(1.0, 4.9 / max(walk, 1.0))

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        _walk_zone(act, me, obs, boost=False, scale=self.scale)
        target = _nearest_mech(obs, me)
        if target is not None:
            act.update(aim=_aim_at(me["pos"], target["pos"]), fire=[True, False])
        return act


class FaceAttackerBot:
    name = "h2-face-atk"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]
        self.last_bearing: float | None = None

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        _walk_zone(act, me, obs)
        for ev in obs.get("events") or []:
            if ev.get("type") == "damaged" and ev.get("bearing") is not None:
                self.last_bearing = float(ev["bearing"])
        target = _nearest_mech(obs, me)
        if target is not None:
            act.update(aim=_aim_at(me["pos"], target["pos"]), fire=[True, True])
        elif self.last_bearing is not None:
            act.update(aim=[self.last_bearing, 0.0], fire=[True, False])
        return act


class SoundTurnBot:
    name = "h2-sound-turn"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]
        self.last_bearing: float | None = None

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        _walk_zone(act, me, obs)
        for snd in obs.get("sounds") or []:
            kind = snd.get("kind")
            if kind in ("gun", "boost", "step", "laser", "lightning", "mortar", "jet") and snd.get("bearing") is not None:
                self.last_bearing = float(snd["bearing"])
        target = _nearest_mech(obs, me)
        if target is not None:
            act.update(aim=_aim_at(me["pos"], target["pos"]), fire=[True, False])
        elif self.last_bearing is not None:
            act.update(aim=[self.last_bearing, 0.0], fire=[True, False])
        return act


class KiteBot:
    name = "h2-kite"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        x, y, z = me["pos"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        target = _nearest_mech(obs, me)
        if target is not None:
            tx, ty, tz = target["pos"]
            away = _toward((tx, ty), (x, y))
            if away == [0.0, 0.0]:
                away = [1.0, 0.0]
            side = [-away[1], away[0]]
            move = [away[0] * 0.7 + side[0] * 0.7, away[1] * 0.7 + side[1] * 0.7]
            n = math.hypot(move[0], move[1]) or 1.0
            act.update(
                move=[move[0] / n, move[1] / n],
                aim=_aim_at(me["pos"], target["pos"]),
                fire=[True, False],
                boost=math.dist(target["pos"], me["pos"]) < 40,
            )
        else:
            _walk_zone(act, me, obs)
        return act


class NearestWeaponBot:
    name = "h2-near-wep"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]
        self.prefer: str | None = None

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach, prefer=self.prefer)
        pickups = list(obs.get("pickups") or [])
        if self.prefer:
            pickups = [p for p in pickups if p.get("kind") == self.prefer] or list(obs.get("pickups") or [])
        target = _nearest_mech(obs, me)
        if pickups and (me.get("weapons") or [None, None])[1] is None:
            goal = min(pickups, key=lambda p: math.dist(_xy(p["pos"]), _xy(me["pos"])))
            move = _toward(_xy(me["pos"]), _xy(goal["pos"]))
            act.update(move=move, aim=[math.atan2(move[1], move[0]), 0.0], boost=True)
        else:
            _walk_zone(act, me, obs)
        if target is not None:
            act.update(aim=_aim_at(me["pos"], target["pos"]), fire=[True, True])
        return act


class TakeLaserBot(NearestWeaponBot):
    name = "h2-take-laser"

    def __init__(self, hello: mf.Hello) -> None:
        super().__init__(hello)
        self.prefer = "laser"


class TakeMortarBot(NearestWeaponBot):
    name = "h2-take-mortar"

    def __init__(self, hello: mf.Hello) -> None:
        super().__init__(hello)
        self.prefer = "mortar"


class TakeLightningBot(NearestWeaponBot):
    name = "h2-take-ltn"

    def __init__(self, hello: mf.Hello) -> None:
        super().__init__(hello)
        self.prefer = "lightning"


class CrateCrashBot:
    name = "h2-crate"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        crates = obs.get("crates") or []
        cubes = obs.get("cubes") or []
        enemy = _nearest_mech(obs, me)
        if crates:
            c = min(crates, key=lambda p: math.dist(p["pos"], me["pos"]))
            act.update(move=_toward(_xy(me["pos"]), _xy(c["pos"])), aim=_aim_at(me["pos"], c["pos"]), fire=[True, True], boost=True)
        elif cubes:
            c = min(cubes, key=lambda p: math.dist(p["pos"], me["pos"]))
            act.update(move=_toward(_xy(me["pos"]), _xy(c["pos"])), boost=True)
            _walk_zone(act, me, obs)
            act["move"] = _toward(_xy(me["pos"]), _xy(c["pos"]))
        else:
            _walk_zone(act, me, obs)
        if enemy is not None:
            act.update(aim=_aim_at(me["pos"], enemy["pos"]), fire=[True, True])
        return act


class JumpJetRoofBot:
    name = "h2-jet-roof"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        _walk_zone(act, me, obs, boost=False)
        energy = float(me.get("energy") or 0)
        grounded = bool(me.get("grounded", True))
        act["jump"] = grounded and energy > 20
        if energy > 25:
            act["boost"] = True
        target = _nearest_mech(obs, me)
        if target is not None:
            act.update(aim=_aim_at(me["pos"], target["pos"]), fire=[True, False])
        return act


class ZoneHugBot:
    name = "h2-zone-hug"

    def __init__(self, hello: mf.Hello) -> None:
        self.reach = hello["config"]["loot"]["pickup_radius"]

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        x, y, z = me["pos"]
        act: mf.Action = {"tick": obs["tick"]}
        _offer_and_take(act, obs, me, self.reach)
        zone = obs.get("zone") or {}
        center = zone.get("center") or zone.get("next_center")
        radius = float(zone.get("radius") or 200)
        if center is None:
            _walk_zone(act, me, obs)
        else:
            cx, cy = center[0], center[1]
            dist = math.hypot(x - cx, y - cy)
            want = max(0.0, radius - 25.0)
            if dist < 1e-3:
                move = [1.0, 0.0]
            elif dist < want:
                move = _toward((cx, cy), (x, y))
            else:
                move = _toward((x, y), (cx, cy))
            act.update(move=move, aim=[math.atan2(move[1], move[0]), 0.0], boost=dist > radius - 8)
        target = _nearest_mech(obs, me)
        if target is not None:
            act.update(aim=_aim_at(me["pos"], target["pos"]), fire=[True, True])
        return act


def _sleep_wrapper(inner_cls, ms: int):
    class SleepBot:
        name = f"sleep-{ms}"[:32]

        def __init__(self, hello: mf.Hello) -> None:
            self.inner = inner_cls(hello)
            self.delay = ms / 1000.0

        def act(self, obs: mf.Observation) -> mf.Action:
            time.sleep(self.delay)
            return self.inner.act(obs)

    return SleepBot


_CONTROL = AimNearestVisibleBot

_MODES = {
    "dump-platform": DumpBot,
    "dump-runtime": DumpRuntimeBot,
    "py-starter": PyStarterBot,
    "aim-nearest-visible": AimNearestVisibleBot,
    "idle": IdleBot,
    "lead-gun": LeadGunBot,
    "boost-center": BoostCenterBot,
    "walk-center": WalkCenterBot,
    "silent-walk": SilentWalkBot,
    "face-attacker": FaceAttackerBot,
    "sound-turn": SoundTurnBot,
    "kite": KiteBot,
    "nearest-weapon": NearestWeaponBot,
    "take-laser": TakeLaserBot,
    "take-mortar": TakeMortarBot,
    "take-lightning": TakeLightningBot,
    "crate-crash": CrateCrashBot,
    "jump-jet-roof": JumpJetRoofBot,
    "zone-hug": ZoneHugBot,
    "sleep-20": _sleep_wrapper(_CONTROL, 20),
    "sleep-35": _sleep_wrapper(_CONTROL, 35),
    "sleep-45": _sleep_wrapper(_CONTROL, 45),
}


def main() -> None:
    mode = _read_probe()
    bot_cls = _MODES.get(mode)
    if bot_cls is None:
        mf.log(f"unknown PROBE mode: {mode!r}")
        sys.exit(2)
    mf.run(bot_cls)


if __name__ == "__main__":
    main()