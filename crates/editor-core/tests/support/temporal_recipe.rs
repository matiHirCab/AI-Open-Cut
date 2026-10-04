//! Closed authored fixture and independent mathematical expectations; no evaluator calls.
use serde_json::{Value, json};

pub const TIMES: [f64; 23] = [
    0.0, 0.5, 99.5, 100.0, 100.5, 299.5, 300.0, 300.5, 499.5, 500.0, 500.5, 713.0, 899.5, 900.0,
    900.5, 950.0, 999.5, 1000.0, 1000.5, 1200.0, 1299.5, 1300.0, 1300.5,
];
pub const COLORS: [&str; 6] = [
    "#ff0000", "#00ff00", "#0000ff", "#ffff00", "#ff00ff", "#00ffff",
];
pub fn bezier(p: f64) -> f64 {
    3.0 * p * p - 2.0 * p * p * p
}
pub fn spring(p: f64) -> f64 {
    let w = 99.0_f64.sqrt();
    1.0 - (-p).exp() * ((w * p).cos() + (w * p).sin() / w)
}
pub fn expected_a(lane: usize, time: f64) -> f64 {
    if lane < 2 {
        return if time <= 0.0 {
            8.0
        } else if time >= 1000.0 {
            20.0
        } else {
            8.0 + 12.0
                * if lane == 0 {
                    spring(time / 1000.0)
                } else {
                    bezier(time / 1000.0)
                }
        };
    }
    if time <= 100.0 || (lane.is_multiple_of(2) && time >= 900.0) {
        return 8.0;
    }
    let phase = (time - 100.0).rem_euclid(400.0);
    let p = if phase <= 200.0 {
        phase / 200.0
    } else {
        (400.0 - phase) / 200.0
    };
    8.0 + 12.0 * if lane < 4 { p } else { bezier(p) }
}
pub fn expected_b(copy: usize, time: f64) -> f64 {
    triangle(0.75 * time + 110.0 - 75.0 * copy as f64)
}
pub fn triangle(source: f64) -> f64 {
    if source <= 100.0 {
        return 8.0;
    }
    let phase = (source - 100.0).rem_euclid(400.0);
    8.0 + 12.0
        * if phase <= 200.0 {
            phase / 200.0
        } else {
            (400.0 - phase) / 200.0
        }
}
pub fn curve() -> Value {
    json!({"type":"cubic_bezier","x1":1.0/3.0,"y1":0,"x2":2.0/3.0,"y2":1})
}
pub fn channel(lane: usize) -> Value {
    let keys = if lane < 2 {
        json!([{"timeMs":0,"value":{"type":"scalar","value":8},"curve":if lane==0 {json!({"type":"spring","mass":1,"stiffness":100,"damping":2,"initialVelocity":0})} else {curve()}},{"timeMs":1000,"value":{"type":"scalar","value":20},"curve":"hold"}])
    } else if lane < 4 {
        json!([{"timeMs":100,"value":{"type":"scalar","value":8},"curve":"linear"},{"timeMs":300,"value":{"type":"scalar","value":20},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":8},"curve":"hold"}])
    } else {
        json!([{"timeMs":100,"value":{"type":"scalar","value":8},"curve":curve()},{"timeMs":300,"value":{"type":"scalar","value":20},"curve":"hold"}])
    };
    let mut c = json!({"property":"transform.position_x","keyframes":keys});
    if lane >= 2 {
        c["loop"] = json!({"mode":if lane<4 {"repeat"} else {"ping_pong"},"iterations":if lane.is_multiple_of(2) {json!(2)} else {json!("infinite")}});
    }
    c
}
pub fn rectangle(id: &str, lane: usize, y: f64) -> Value {
    json!({"type":"rectangle","id":id,"startMs":0,"durationMs":1300,"width":5,"height":5,"color":COLORS[lane],"transform":{"positionX":8,"positionY":y,"scale":1,"opacity":1},"keyframes":[],"animationChannels":[channel(lane)],"zIndex":0,"stackOrder":lane})
}
pub fn track(items: Value) -> Value {
    json!({"id":"local","name":"Temporal lanes","trackType":"overlay","items":items})
}
pub fn family_a() -> Value {
    json!([track(Value::Array(
        (0..6)
            .map(|i| rectangle(&format!("lane{i}"), i, 5.0 + 9.0 * i as f64))
            .collect()
    ))])
}
pub fn family_b(leaf: &str, outer: &str) -> (Value, Value, Value) {
    let mut visible = rectangle("visible", 3, 8.0);
    visible["durationMs"] = json!(1200);
    visible["stackOrder"] = json!(1);
    visible["animationChannels"][0]["clock"] = json!({"offsetMs":250,"sourceDurationMs":1500});
    let mut hidden = visible.clone();
    hidden["id"] = json!("hidden-leaf");
    hidden["hidden"] = json!(true);
    hidden["stackOrder"] = json!(0);
    hidden["animationChannels"] = json!([]);
    let leaf_tracks = json!([track(json!([hidden, visible]))]);
    let parent = json!({"scope":format!("component:{outer}"),"id":"group"});
    let inner = json!({"type":"component_instance","id":"inner","componentId":leaf,"startMs":20,"trimStartMs":10,"durationMs":1900,"timeScale":0.5,"staggerMs":40,"slotValues":{},"parent":parent,"zIndex":0,"stackOrder":2});
    let mut hidden = rectangle("hidden-outer", 3, 0.0);
    hidden["hidden"] = json!(true);
    hidden["durationMs"] = json!(1900);
    hidden["animationChannels"] = json!([]);
    hidden["parent"] = parent;
    hidden["stackOrder"] = json!(1);
    let group = json!({"type":"group","id":"group","startMs":0,"durationMs":2000,"staggerMs":100,"zIndex":0,"stackOrder":0});
    let outer_tracks = json!([track(json!([group, hidden, inner]))]);
    let root = json!([{"type":"component_instance","id":"source","componentId":outer,"startMs":100,"trimStartMs":50,"durationMs":1200,"timeScale":1.5,"slotValues":{},"zIndex":0,"stackOrder":0},{"type":"repeater","id":"copies","startMs":0,"durationMs":1300,"zIndex":0,"stackOrder":1,"repeater":{"source":{"scope":"root","id":"source"},"copies":2,"timeOffsetMs":100,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":16,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}]);
    (leaf_tracks, outer_tracks, root)
}
