pub use web_time::*;

pub fn now() -> f64 {
    rightnow::RightNow::now().as_secs_f64() * 1000.0
}
