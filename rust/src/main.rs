use serde_json::{json, Value};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

fn read_probe() -> String {
    if let Ok(p) = std::env::var("PROBE") {
        let t = p.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    for path in [
        Path::new("/bot/PROBE"),
        Path::new("PROBE"),
        Path::new("/PROBE"),
    ] {
        if let Ok(s) = fs::read_to_string(path) {
            if let Some(line) = s.lines().next() {
                let t = line.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
    }
    "aim-nearest-visible".into()
}

fn f64_at(v: &Value, i: usize) -> f64 {
    v.as_array()
        .and_then(|a| a.get(i))
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
}

fn vec2(v: &Value) -> [f64; 2] {
    [f64_at(v, 0), f64_at(v, 1)]
}

fn vec3(v: &Value) -> [f64; 3] {
    [f64_at(v, 0), f64_at(v, 1), f64_at(v, 2)]
}

fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

fn dist3(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn toward(src: [f64; 2], dst: [f64; 2]) -> [f64; 2] {
    let dx = dst[0] - src[0];
    let dy = dst[1] - src[1];
    let n = (dx * dx + dy * dy).sqrt();
    if n < 1e-6 {
        return [0.0, 0.0];
    }
    [dx / n, dy / n]
}

fn aim_at(me: [f64; 3], target: [f64; 3]) -> [f64; 2] {
    let yaw = (target[1] - me[1]).atan2(target[0] - me[0]);
    let horiz = dist2([target[0], target[1]], [me[0], me[1]]);
    let pitch = (target[2] - me[2] - 2.5).atan2(horiz);
    [yaw, pitch]
}

fn zone_goal(obs: &Value) -> Option<[f64; 2]> {
    let zone = obs.get("zone")?;
    zone.get("next_center")
        .or_else(|| zone.get("center"))
        .map(vec2)
}

fn nearest_mech<'a>(obs: &'a Value, me_pos: [f64; 3]) -> Option<&'a Value> {
    let mechs = obs.get("mechs")?.as_array()?;
    mechs
        .iter()
        .filter(|m| m.get("kind").and_then(Value::as_str) == Some("mech"))
        .min_by(|a, b| {
            let da = dist3(vec3(a.get("pos").unwrap_or(&Value::Null)), me_pos);
            let db = dist3(vec3(b.get("pos").unwrap_or(&Value::Null)), me_pos);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn gun_speed(me: &Value) -> f64 {
    me.get("weapons")
        .and_then(Value::as_array)
        .and_then(|w| w.first())
        .and_then(|w0| w0.get("stats"))
        .and_then(|s| s.get("speed"))
        .and_then(Value::as_f64)
        .unwrap_or(252.0)
        .max(1.0)
}

fn lead_pos(me: &Value, target: &Value) -> [f64; 3] {
    let me_pos = vec3(me.get("pos").unwrap_or(&Value::Null));
    let tpos = vec3(target.get("pos").unwrap_or(&Value::Null));
    let tvel = vec3(target.get("vel").unwrap_or(&Value::Null));
    let t = dist3(tpos, me_pos) / gun_speed(me);
    [tpos[0] + tvel[0] * t, tpos[1] + tvel[1] * t, tpos[2] + tvel[2] * t]
}

fn walk_zone(me: &Value, obs: &Value) -> (Vec<f64>, Vec<f64>) {
    let pos = vec3(me.get("pos").unwrap_or(&Value::Null));
    if let Some(goal) = zone_goal(obs) {
        let yaw = (goal[1] - pos[1]).atan2(goal[0] - pos[0]);
        (vec![yaw.cos(), yaw.sin()], vec![yaw, 0.0])
    } else {
        let aim = me.get("aim").map(vec2).unwrap_or([0.0, 0.0]);
        (vec![0.0, 0.0], vec![aim[0], aim[1]])
    }
}

fn pip(x: f64, y: f64, ring: &[f64]) -> bool {
    let n = ring.len() / 2;
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let xi = ring[2 * i];
        let yi = ring[2 * i + 1];
        let xj = ring[2 * j];
        let yj = ring[2 * j + 1];
        let dy = yj - yi;
        let dy = if dy == 0.0 { 1e-12 } else { dy };
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / dy + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn floor_div(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0;
    }
    let (aa, bb) = (a, b);
    let q = aa / bb;
    let r = aa % bb;
    if r != 0 && (aa < 0) != (bb < 0) {
        q - 1
    } else {
        q
    }
}

struct OccupancyGrid {
    minx: f64,
    miny: f64,
    step: f64,
    nx: usize,
    ny: usize,
    grid: Vec<u8>,
}

impl OccupancyGrid {
    fn new(world: &Value, step: f64) -> Option<Self> {
        let b = world.get("bounds")?;
        let minx = b.get("minX")?.as_f64()?;
        let miny = b.get("minY")?.as_f64()?;
        let maxx = b.get("maxX")?.as_f64()?;
        let maxy = b.get("maxY")?.as_f64()?;
        let nx = ((maxx - minx) / step).ceil() as usize + 1;
        let ny = ((maxy - miny) / step).ceil() as usize + 1;
        let mut grid = vec![0u8; nx * ny];
        let empty: Vec<Value> = vec![];
        let prisms = world.get("prisms").and_then(Value::as_array).unwrap_or(&empty);
        for pr in prisms {
            let base = pr.get("base").and_then(Value::as_f64).unwrap_or(0.0);
            let top = pr.get("top").and_then(Value::as_f64).unwrap_or(0.0);
            if base >= 2.5 || top <= 1.2 {
                continue;
            }
            let ring_v = pr.get("ring")?;
            let ring: Vec<f64> = ring_v
                .as_array()?
                .iter()
                .filter_map(Value::as_f64)
                .collect();
            if ring.len() < 6 {
                continue;
            }
            let xs: Vec<f64> = ring.iter().step_by(2).copied().collect();
            let ys: Vec<f64> = ring.iter().skip(1).step_by(2).copied().collect();
            let i0 = ((xs.iter().copied().fold(f64::INFINITY, f64::min) - minx) / step)
                .floor() as i32;
            let i0 = i0.saturating_sub(1).max(0) as usize;
            let i1 = ((xs.iter().copied().fold(f64::NEG_INFINITY, f64::max) - minx) / step)
                .floor() as i32
                + 1;
            let i1 = (i1 as usize).min(nx - 1);
            let j0 = ((ys.iter().copied().fold(f64::INFINITY, f64::min) - miny) / step)
                .floor() as i32;
            let j0 = j0.saturating_sub(1).max(0) as usize;
            let j1 = ((ys.iter().copied().fold(f64::NEG_INFINITY, f64::max) - miny) / step)
                .floor() as i32
                + 1;
            let j1 = (j1 as usize).min(ny - 1);
            let holes: Vec<Vec<f64>> = pr
                .get("holes")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|h| {
                            h.as_array().map(|a| {
                                a.iter().filter_map(Value::as_f64).collect::<Vec<f64>>()
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            for j in j0..=j1 {
                let y = miny + (j as f64 + 0.5) * step;
                let row = j * nx;
                for i in i0..=i1 {
                    let x = minx + (i as f64 + 0.5) * step;
                    if pip(x, y, &ring)
                        && !holes.iter().any(|h| pip(x, y, h))
                    {
                        grid[row + i] = 1;
                    }
                }
            }
        }
        Some(Self {
            minx,
            miny,
            step,
            nx,
            ny,
            grid,
        })
    }

    fn blocked_count(&self) -> usize {
        self.grid.iter().filter(|&&c| c != 0).count()
    }

    fn ij(&self, x: f64, y: f64) -> (usize, usize) {
        let i = ((x - self.minx) / self.step).floor() as i32;
        let j = ((y - self.miny) / self.step).floor() as i32;
        let i = i.clamp(0, self.nx as i32 - 1) as usize;
        let j = j.clamp(0, self.ny as i32 - 1) as usize;
        (i, j)
    }

    fn step_toward(&self, src: [f64; 3], dst: [f64; 2]) -> Option<[f64; 2]> {
        let (si, sj) = self.ij(src[0], src[1]);
        let (gi, gj) = self.ij(dst[0], dst[1]);
        let nx = self.nx;
        let ny = self.ny;
        let grid = &self.grid;
        if grid[sj * nx + si] != 0 || grid[gj * nx + gi] != 0 {
            return None;
        }
        let h = |i: usize, j: usize| ((i as i32 - gi as i32).abs() + (j as i32 - gj as i32).abs()) as f64;
        let mut openh: BinaryHeap<Reverse<(OrderedFloat, OrderedFloat, usize, usize)>> =
            BinaryHeap::new();
        openh.push(Reverse((
            OrderedFloat(h(si, sj)),
            OrderedFloat(0.0),
            si,
            sj,
        )));
        let mut gbest = vec![f64::INFINITY; nx * ny];
        gbest[sj * nx + si] = 0.0;
        let mut came: Vec<Option<usize>> = vec![None; nx * ny];
        let mut found = false;
        let mut popped = 0usize;
        while let Some(Reverse((_, OrderedFloat(g), i, j))) = openh.pop() {
            popped += 1;
            if popped >= 8000 {
                break;
            }
            if i == gi && j == gj {
                found = true;
                break;
            }
            if g > gbest[j * nx + i] {
                continue;
            }
            const NEIGH: [(i32, i32); 8] = [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ];
            for (di, dj) in NEIGH {
                let ni = i as i32 + di;
                let nj = j as i32 + dj;
                if ni < 0 || nj < 0 || ni >= nx as i32 || nj >= ny as i32 {
                    continue;
                }
                let (ni, nj) = (ni as usize, nj as usize);
                let nidx = nj * nx + ni;
                if grid[nidx] != 0 {
                    continue;
                }
                let ng = g + if di != 0 && dj != 0 { 1.414 } else { 1.0 };
                if ng < gbest[nidx] {
                    gbest[nidx] = ng;
                    came[nidx] = Some(j * nx + i);
                    openh.push(Reverse((
                        OrderedFloat(ng + h(ni, nj)),
                        OrderedFloat(ng),
                        ni,
                        nj,
                    )));
                }
            }
        }
        if !found {
            return None;
        }
        let mut path = vec![gj * nx + gi];
        let mut cur = path[0];
        while let Some(prev) = came[cur] {
            cur = prev;
            path.push(cur);
        }
        path.reverse();
        let look = path[usize::min(4, path.len().saturating_sub(1))];
        let ti = look % nx;
        let tj = look / nx;
        let tx = self.minx + (ti as f64 + 0.5) * self.step;
        let ty = self.miny + (tj as f64 + 0.5) * self.step;
        Some(toward([src[0], src[1]], [tx, ty]))
    }

    fn cover_step(&self, src: [f64; 3], threat_xy: [f64; 2], ring: i32) -> Option<[f64; 2]> {
        let (si, sj) = self.ij(src[0], src[1]);
        let (ti, tj) = self.ij(threat_xy[0], threat_xy[1]);
        let nx = self.nx;
        let ny = self.ny;
        let grid = &self.grid;
        let mut best: Option<(usize, usize)> = None;
        let mut best_d = i32::MAX;
        for r in 2..=ring {
            for di in -r..=r {
                for dj in -r..=r {
                    if di.abs() != r && dj.abs() != r {
                        continue;
                    }
                    let ni = si as i32 + di;
                    let nj = sj as i32 + dj;
                    if ni < 0 || nj < 0 || ni >= nx as i32 || nj >= ny as i32 {
                        continue;
                    }
                    let (ni, nj) = (ni as usize, nj as usize);
                    if grid[nj * nx + ni] != 0 {
                        continue;
                    }
                    let n = (ti as i32 - ni as i32)
                        .abs()
                        .max((tj as i32 - nj as i32).abs())
                        .max(1);
                    let mut blocked = false;
                    for k in 1..n {
                        let ii = ni as i32 + floor_div((ti as i32 - ni as i32) * k, n);
                        let jj = nj as i32 + floor_div((tj as i32 - nj as i32) * k, n);
                        if ii >= 0
                            && jj >= 0
                            && (ii as usize) < nx
                            && (jj as usize) < ny
                            && grid[jj as usize * nx + ii as usize] != 0
                        {
                            blocked = true;
                            break;
                        }
                    }
                    if !blocked {
                        continue;
                    }
                    let d = di.abs() + dj.abs();
                    if d < best_d {
                        best_d = d;
                        best = Some((ni, nj));
                    }
                }
            }
            if best.is_some() {
                break;
            }
        }
        let (bi, bj) = best?;
        let tx = self.minx + (bi as f64 + 0.5) * self.step;
        let ty = self.miny + (bj as f64 + 0.5) * self.step;
        self.step_toward(src, [tx, ty])
            .or_else(|| Some(toward([src[0], src[1]], [tx, ty])))
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

struct MapIntel {
    loaded: bool,
    load_ms: f64,
    n_prisms: usize,
    grid_ms: Option<f64>,
    blocked: Option<usize>,
}

fn world_json_candidates(hello: &Value) -> Vec<String> {
    let mut cands = Vec::new();
    if let Some(d) = hello
        .get("map")
        .and_then(|m| m.get("dir"))
        .and_then(Value::as_str)
    {
        cands.push(format!("{}/v2/world.json", d));
        cands.push(format!("{}/world.json", d));
    }
    cands.push("/arena/v2/world.json".into());
    cands.push("/arena/complete/world.json".into());
    cands.push("mechfight-kit/arena/v2/world.json".into());
    cands
}

fn load_map(hello: &Value) -> (MapIntel, Option<OccupancyGrid>) {
    let t0 = Instant::now();
    for path in world_json_candidates(hello) {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                let n = v
                    .get("prisms")
                    .and_then(Value::as_array)
                    .map(|a| a.len())
                    .unwrap_or(0);
                let load_ms = t0.elapsed().as_secs_f64() * 1000.0;
                let mut grid_ms = None;
                let mut blocked = None;
                let grid = if n > 0 {
                    let tg = Instant::now();
                    let g = OccupancyGrid::new(&v, 4.0);
                    grid_ms = Some(tg.elapsed().as_secs_f64() * 1000.0);
                    if let Some(ref gr) = g {
                        blocked = Some(gr.blocked_count());
                    }
                    g
                } else {
                    None
                };
                return (
                    MapIntel {
                        loaded: true,
                        load_ms,
                        n_prisms: n,
                        grid_ms,
                        blocked,
                    },
                    grid,
                );
            }
        }
    }
    (
        MapIntel {
            loaded: false,
            load_ms: t0.elapsed().as_secs_f64() * 1000.0,
            n_prisms: 0,
            grid_ms: None,
            blocked: None,
        },
        None,
    )
}

struct Bot {
    probe: String,
    name: String,
    map: MapIntel,
    grid: Option<OccupancyGrid>,
    last_bearing: Option<f64>,
    hurt_until: f64,
}

impl Bot {
    fn new(probe: String, hello: &Value) -> Self {
        let (map, grid) = load_map(hello);
        let name = match probe.as_str() {
            "idle" => "h2-rs-idle",
            "lead-gun" => "h2-rs-lead",
            "path-zone" => "h3-rs-path-zone",
            "hide-prism" => "h3-rs-hide-prism",
            _ => "h2-rs-aim-nv",
        }
        .to_string();
        Self {
            probe,
            name,
            map,
            grid,
            last_bearing: None,
            hurt_until: -1.0,
        }
    }

    fn act(&mut self, obs: &Value) -> Value {
        let me = obs.get("self").cloned().unwrap_or(json!({}));
        let tick = obs.get("tick").cloned().unwrap_or(json!(0));
        match self.probe.as_str() {
            "idle" => {
                let aim = me.get("aim").cloned().unwrap_or(json!([0.0, 0.0]));
                json!({
                    "tick": tick,
                    "move": [0.0, 0.0],
                    "aim": aim,
                    "boost": false,
                    "jump": false
                })
            }
            "path-zone" => self.act_path_zone(&me, obs, tick),
            "hide-prism" => self.act_hide_prism(&me, obs, tick),
            "lead-gun" => self.chase(&me, obs, tick, true),
            _ => self.chase(&me, obs, tick, false),
        }
    }

    fn act_path_zone(&mut self, me: &Value, obs: &Value, tick: Value) -> Value {
        let me_pos = vec3(me.get("pos").unwrap_or(&Value::Null));
        if let Some(target) = nearest_mech(obs, me_pos) {
            let tgt = vec3(target.get("pos").unwrap_or(&Value::Null));
            let [yaw, pitch] = aim_at(me_pos, tgt);
            return json!({
                "tick": tick,
                "move": [yaw.cos(), yaw.sin()],
                "aim": [yaw, pitch],
                "fire": [true, false]
            });
        }
        if let Some(ref grid) = self.grid {
            if let Some(goal) = zone_goal(obs) {
                if let Some(step) = grid.step_toward(me_pos, goal) {
                    return json!({
                        "tick": tick,
                        "move": [step[0], step[1]],
                        "aim": [step[1].atan2(step[0]), 0.0]
                    });
                }
            }
        }
        let (mv, aim) = walk_zone(me, obs);
        json!({ "tick": tick, "move": mv, "aim": aim })
    }

    fn act_hide_prism(&mut self, me: &Value, obs: &Value, tick: Value) -> Value {
        let me_pos = vec3(me.get("pos").unwrap_or(&Value::Null));
        let t = obs.get("t").and_then(Value::as_f64).unwrap_or(0.0);
        let x = me_pos[0];
        let y = me_pos[1];
        if let Some(events) = obs.get("events").and_then(Value::as_array) {
            for ev in events {
                if ev.get("type").and_then(Value::as_str) == Some("damaged") {
                    if let Some(br) = ev.get("bearing").and_then(Value::as_f64) {
                        self.last_bearing = Some(br);
                        self.hurt_until = t + 4.0;
                    }
                }
            }
        }
        if t < self.hurt_until {
            if let Some(br) = self.last_bearing {
                let threat = [x + 80.0 * br.cos(), y + 80.0 * br.sin()];
                let step = self
                    .grid
                    .as_ref()
                    .and_then(|g| g.cover_step(me_pos, threat, 10));
                let step = step.unwrap_or_else(|| {
                    let away = br + std::f64::consts::PI;
                    [away.cos(), away.sin()]
                });
                return json!({
                    "tick": tick,
                    "move": [step[0], step[1]],
                    "aim": [br, 0.0],
                    "boost": true,
                    "fire": [false, false]
                });
            }
        }
        if let Some(target) = nearest_mech(obs, me_pos) {
            let lead = lead_pos(me, target);
            let [yaw, pitch] = aim_at(me_pos, lead);
            return json!({
                "tick": tick,
                "move": [yaw.cos(), yaw.sin()],
                "aim": [yaw, pitch],
                "fire": [true, false]
            });
        }
        let (mv, aim) = walk_zone(me, obs);
        json!({ "tick": tick, "move": mv, "aim": aim })
    }

    fn chase(&self, me: &Value, obs: &Value, tick: Value, lead: bool) -> Value {
        let me_pos = vec3(me.get("pos").unwrap_or(&Value::Null));
        if let Some(target) = nearest_mech(obs, me_pos) {
            let tgt = if lead {
                lead_pos(me, target)
            } else {
                vec3(target.get("pos").unwrap_or(&Value::Null))
            };
            let [yaw, pitch] = aim_at(me_pos, tgt);
            json!({
                "tick": tick,
                "move": [yaw.cos(), yaw.sin()],
                "aim": [yaw, pitch],
                "fire": [true, false]
            })
        } else {
            let (mv, aim) = walk_zone(me, obs);
            json!({ "tick": tick, "move": mv, "aim": aim })
        }
    }
}

fn send(out: &mut impl Write, v: &Value) {
    let _ = writeln!(out, "{}", v);
    let _ = out.flush();
}

fn main() {
    let probe = read_probe();
    let (tx, rx) = mpsc::channel::<Option<String>>();
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) if !l.trim().is_empty() => {
                    if tx.send(Some(l)).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => {
                    let _ = tx.send(None);
                    break;
                }
            }
        }
        let _ = tx.send(None);
    });

    let mut bot: Option<Bot> = None;
    let mut last = json!({});
    let mut out = io::stdout();

    loop {
        let first = match rx.recv() {
            Ok(Some(l)) => l,
            Ok(None) | Err(_) => return,
        };
        let mut batch = vec![first];
        while let Ok(more) = rx.try_recv() {
            match more {
                Some(l) => batch.push(l),
                None => return,
            }
        }

        let mut latest: Option<Value> = None;
        for raw in batch {
            let msg: Value = match serde_json::from_str(&raw) {
                Ok(v) => v,
                Err(_) => continue,
            };
            match msg.get("type").and_then(Value::as_str) {
                Some("hello") => {
                    let b = Bot::new(probe.clone(), &msg);
                    send(
                        &mut out,
                        &json!({"type": "ready", "name": b.name}),
                    );
                    let mut line = format!(
                        "map_loaded={} load_ms={:.2} prisms={}",
                        b.map.loaded, b.map.load_ms, b.map.n_prisms
                    );
                    if let Some(ms) = b.map.grid_ms {
                        line.push_str(&format!(" grid_ms={:.2}", ms));
                    }
                    if let Some(bl) = b.map.blocked {
                        line.push_str(&format!(" blocked={}", bl));
                    }
                    let _ = writeln!(io::stderr(), "{}", line);
                    bot = Some(b);
                }
                Some("obs") => latest = Some(msg),
                Some("end") => return,
                _ => {}
            }
        }
        let Some(obs) = latest else { continue };
        let tick = obs.get("tick").cloned().unwrap_or(json!(0));
        let mut action = if let Some(b) = bot.as_mut() {
            b.act(&obs)
        } else {
            json!({"tick": tick})
        };
        if let Some(obj) = action.as_object() {
            let mut keep = serde_json::Map::new();
            for k in ["move", "aim", "boost"] {
                if let Some(v) = obj.get(k) {
                    keep.insert(k.to_string(), v.clone());
                }
            }
            last = Value::Object(keep);
        }
        if let Some(obj) = action.as_object_mut() {
            obj.insert("tick".into(), tick);
        }
        send(&mut out, &action);
        let _ = last;
    }
}