//! Shared viewport constraints for native surfaces, including a visible software keyboard.
use reader_state::AppViewModel;

pub const NARROW: f64 = 700.0;
pub const TOUCH_TARGET: f64 = 44.0;

pub fn narrow(vm: &AppViewModel) -> bool {
    vm.viewport() < NARROW
}
pub fn touch(vm: &AppViewModel) -> bool {
    narrow(vm) || vm.touch_mode()
}

pub struct DialogMetrics {
    pub width: f64,
    pub height: f64,
    pub padding: (f64, f64),
    pub gap: f64,
    pub body_height: f64,
}

impl DialogMetrics {
    pub fn new(vm: &AppViewModel, preferred_width: f64, preferred_height: f64) -> Self {
        let small = narrow(vm);
        let inset = if small { 8.0 } else { 16.0 };
        let padding = if small || vm.viewport_height() < 450.0 {
            (16.0, 16.0)
        } else {
            (42.0, 30.0)
        };
        let gap = if small { 12.0 } else { 16.0 };
        let width = preferred_width.min((vm.viewport() - 2.0 * inset).max(1.0));
        let height = preferred_height.min((vm.viewport_height() - 2.0 * inset).max(1.0));
        let body_height = (height - padding.1 * 2.0 - 24.0 - gap - 2.0).max(1.0);
        Self {
            width,
            height,
            padding,
            gap,
            body_height,
        }
    }
}
