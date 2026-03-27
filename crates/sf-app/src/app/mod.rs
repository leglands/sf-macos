//! AppKit application entry point (objc2 0.5 API with MainThreadMarker)

mod state;
mod window;

use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;
pub use state::AppState;

pub fn run() {
    // MainThreadMarker — asserts we're on the main thread (required by objc2 0.5 for AppKit)
    let mtm = MainThreadMarker::new().expect("must run on main thread");

    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    let _window = window::create_main_window(mtm);

    unsafe { app.run() };
}
