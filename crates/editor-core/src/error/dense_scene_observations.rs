//! Request-local test evidence. No production build, feature or wire surface.
use super::{RenderBudget, RenderBudgetDiagnostic};
use std::{cell::RefCell, time::Instant};

#[derive(Default)]
struct State {
    active: bool,
    charges: Vec<RenderBudgetDiagnostic>,
    timings: Vec<(&'static str, u64, f64)>,
}
thread_local! { static STATE: RefCell<State> = RefCell::default(); }

pub(crate) struct Session;
impl Session {
    pub(crate) fn start() -> Self {
        STATE.with_borrow_mut(|s| {
            assert!(!s.active, "nested measurement session");
            *s = State {
                active: true,
                ..Default::default()
            };
        });
        Self
    }
    pub(crate) fn finish(self) -> serde_json::Value {
        STATE.with_borrow(|s| serde_json::json!({
            "ownerChargeMaxima": s.charges,
            "directInclusiveTimings": s.timings.iter().map(|(name,calls,ms)|
                serde_json::json!({"name":name,"calls":calls,"milliseconds":ms})).collect::<Vec<_>>()
        }))
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        STATE.with_borrow_mut(|s| *s = State::default());
    }
}
pub(crate) fn charge(budget: RenderBudget, limit: u64, value: u64) {
    STATE.with_borrow_mut(|s| {
        if !s.active {
            return;
        }
        if let Some(c) = s.charges.iter_mut().find(|c| c.budget == budget) {
            assert_eq!(c.limit, limit.to_string());
            if value > c.work.as_ref().unwrap().parse::<u64>().unwrap() {
                c.work = Some(value.to_string());
            }
        } else {
            assert!(s.charges.len() < 8);
            s.charges.push(RenderBudgetDiagnostic {
                budget,
                limit: limit.to_string(),
                work: Some(value.to_string()),
                units: budget.units(),
                measurement: budget.measurement(),
                overflow: false,
                contributors: vec![],
            });
        }
    });
}
pub(crate) fn elapsed(name: &'static str, milliseconds: f64) {
    assert!(milliseconds.is_finite() && milliseconds >= 0.);
    STATE.with_borrow_mut(|s| {
        if !s.active {
            return;
        }
        if let Some((_, calls, ms)) = s.timings.iter_mut().find(|(n, _, _)| *n == name) {
            *calls += 1;
            *ms += milliseconds;
        } else {
            assert!(s.timings.len() < 3);
            s.timings.push((name, 1, milliseconds));
        }
    });
}
pub(crate) fn valid_charge(value: &serde_json::Value) -> bool {
    use RenderBudget::*;
    let Some(budget) = [
        ShapeSegments,
        SceneShapeSegments,
        SceneShapeRasterBytes,
        CandidateAnalysisNodes,
        EffectPixelPasses,
        MotionBlurPixelWork,
        CompositionPixelWork,
        CompositionBytes,
    ]
    .into_iter()
    .find(|b| serde_json::to_value(b).unwrap() == value["budget"]) else {
        return false;
    };
    let Some(work) = value["work"].as_str().and_then(|s| s.parse::<u64>().ok()) else {
        return false;
    };
    let Some(limit) = value["limit"].as_str().and_then(|s| s.parse::<u64>().ok()) else {
        return false;
    };
    work <= limit
        && value["overflow"] == false
        && value["contributors"] == serde_json::json!([])
        && value["units"] == budget.units()
        && value["measurement"] == budget.measurement()
        && limit
            == match budget {
                ShapeSegments | CandidateAnalysisNodes => 65536,
                SceneShapeSegments => 1048576,
                SceneShapeRasterBytes => 16777216 * 12 * 4096,
                EffectPixelPasses | MotionBlurPixelWork | CompositionPixelWork => 268435456,
                CompositionBytes => 1073741824,
            }
}
pub(crate) struct InclusiveTimer(&'static str, Instant);
impl InclusiveTimer {
    pub(crate) fn start(name: &'static str) -> Option<Self> {
        STATE.with_borrow(|s| s.active.then(|| Self(name, Instant::now())))
    }
}
impl Drop for InclusiveTimer {
    fn drop(&mut self) {
        elapsed(self.0, self.1.elapsed().as_secs_f64() * 1000.);
    }
}

#[test]
fn observations_are_bounded_request_local_owner_facts_and_unwind_safe() {
    let s = Session::start();
    charge(RenderBudget::CompositionPixelWork, 268435456, 10);
    charge(RenderBudget::CompositionPixelWork, 268435456, 3);
    charge(RenderBudget::CompositionBytes, 1073741824, 100);
    elapsed("source_rasterization", 2.);
    elapsed("source_rasterization", 3.);
    let report = s.finish();
    assert_eq!(report["ownerChargeMaxima"][0]["work"], "10");
    assert_eq!(
        report["ownerChargeMaxima"][1]["measurement"],
        "conservative"
    );
    assert_eq!(report["directInclusiveTimings"][0]["calls"], 2);
    assert_eq!(report["directInclusiveTimings"][0]["milliseconds"], 5.);
    let _ = std::panic::catch_unwind(|| {
        let _s = Session::start();
        panic!("injected request unwind")
    });
    let empty = Session::start().finish();
    assert_eq!(empty["ownerChargeMaxima"], serde_json::json!([]));
    assert_eq!(empty["directInclusiveTimings"], serde_json::json!([]));
}
