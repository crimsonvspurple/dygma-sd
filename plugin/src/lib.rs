//! Library surface for the Stream Deck plugin and tooling (e.g. marketplace asset gen).
//!
//! The plugin binary lives in `main.rs`. Marketplace PNGs are produced by
//! `cargo run --features gen-marketplace --bin gen-marketplace`.

pub mod battery;
pub mod error;
pub mod plugin;
pub mod visual;
