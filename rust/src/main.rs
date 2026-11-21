use serde_json::{json, Value};
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

struct MapIntel {
    loaded: bool,
    load_ms: f64,
    n_prisms: usize,
}

fn load_map() -> MapIntel {
    let t0 = Instant::now();
    for path in [
        "/arena/v2/world.json",
        "/arena/complete/world.json",
        "mechfight-kit/arena/v2/world.json",
    ] {
        if let Ok(raw) = fs::read_to_string(path) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                let n = v
                    .get("prisms")
                    .and_then(Value::as_array)
                    .map(|a| a.len())
                    .unwrap_or(0);
                return MapIntel {
                    loaded: true,
                    load_ms: t0.elapsed().as_secs_f64() * 1000.0,
                    n_prisms: n,
                };
            }
        }
    }
    MapIntel {
        loaded: false,
        load_ms: t0.elapsed().as_secs_f64() * 1000.0,
        n_prisms: 0,
    }
}

struct Bot {
    probe: String,
    name: String,
    map: MapIntel,
}

impl Bot {
    fn new(probe: String) -> Self {
        let map = load_map();
        let name = match probe.as_str() {
            "idle" => "h2-rs-idle",
            "lead-gun" => "h2-rs-lead",
            _ => "h2-rs-aim-nv",
        }
        .to_string();
        Self { probe, name, map }
    }

    fn act(&self, obs: &Value) -> Value {
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
            "lead-gun" => self.chase(&me, obs, tick, true),
            _ => self.chase(&me, obs, tick, false),
        }
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
                    let b = Bot::new(probe.clone());
                    send(
                        &mut out,
                        &json!({"type": "ready", "name": b.name}),
                    );
                    let _ = writeln!(
                        io::stderr(),
                        "map_loaded={} load_ms={:.2} prisms={}",
                        b.map.loaded,
                        b.map.load_ms,
                        b.map.n_prisms
                    );
                    bot = Some(b);
                }
                Some("obs") => latest = Some(msg),
                Some("end") => return,
                _ => {}
            }
        }
        let Some(obs) = latest else { continue };
        let tick = obs.get("tick").cloned().unwrap_or(json!(0));
        let mut action = if let Some(b) = bot.as_ref() {
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
