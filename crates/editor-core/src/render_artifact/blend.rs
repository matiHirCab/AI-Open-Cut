//! Fixed pure linear premultiplied blend equations, after canonical f32 admission.
use crate::BlendMode;
/// Inputs have passed the canonical f32 boundary and have been clamped in f32.
/// No allocations, callbacks, or fallible operations enter the commit path.
pub(super) fn composite(destination: &mut [f32; 4], source: [f32; 4], mode: BlendMode) {
    debug_assert!(!mode.is_normal());
    let sa = f64::from(source[3]);
    let da = f64::from(destination[3]);
    let alpha = sa + da - sa * da;
    for c in 0..3 {
        let sp = f64::from(source[c]);
        let dp = f64::from(destination[c]);
        let s = if sa > 0.0 { sp / sa } else { 0.0 };
        let d = if da > 0.0 { dp / da } else { 0.0 };
        let b = match mode {
            BlendMode::Multiply => s * d,
            BlendMode::Screen => s + d - s * d,
            BlendMode::Overlay => {
                if d <= 0.5 {
                    2.0 * s * d
                } else {
                    1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                }
            }
            BlendMode::Add => (s + d).min(1.0),
            BlendMode::Darken => s.min(d),
            BlendMode::Lighten => s.max(d),
            BlendMode::Normal => unreachable!("normal uses the unchanged f32 compositor"),
        };
        destination[c] = ((1.0 - sa) * dp + (1.0 - da) * sp + sa * da * b).clamp(0.0, alpha) as f32;
    }
    destination[3] = alpha.clamp(0.0, 1.0) as f32;
}
