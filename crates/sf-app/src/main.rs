//! Software Factory — macOS native app
//! AppKit UI via objc2, Full Rust, zero runtime dependencies
//! Connects to local Ollama + remote SF instances (OVH, Azure)

use std::sync::Arc;
use tracing_subscriber::EnvFilter;

mod app;

fn main() {
    // Init logging (RUST_LOG=info for verbose)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("sf_app=info".parse().unwrap()))
        .init();

    tracing::info!("Software Factory macOS — starting");

    app::run();
}
