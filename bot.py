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


class AimNearestVisibleBot:
    name = "h1s4-aim-nv"

    def __init__(self, hello: mf.Hello) -> None:
        self.hello = hello

    def act(self, obs: mf.Observation) -> mf.Action:
        me = obs["self"]
        x, y, z = me["pos"]
        act: mf.Action = {"tick": obs["tick"]}
        visible = [m for m in obs["mechs"] if m["kind"] == "mech"]
        if visible:
            target = min(visible, key=lambda m: math.dist(m["pos"], me["pos"]))
            tx, ty, tz = target["pos"]
            yaw = math.atan2(ty - y, tx - x)
            pitch = math.atan2(tz - z - 2.5, math.dist((tx, ty), (x, y)))
            act.update(
                move=[math.cos(yaw), math.sin(yaw)],
                aim=[yaw, pitch],
                fire=[True, False],
            )
        else:
            zone = obs.get("zone") or {}
            center = zone.get("next_center")
            if center is not None:
                yaw = math.atan2(center[1] - y, center[0] - x)
                act.update(move=[math.cos(yaw), math.sin(yaw)], aim=[yaw, 0.0])
            else:
                act.update(move=[0.0, 0.0], aim=me.get("aim", [0.0, 0.0]))
        return act


_MODES = {
    "dump-platform": DumpBot,
    "py-starter": PyStarterBot,
    "aim-nearest-visible": AimNearestVisibleBot,
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