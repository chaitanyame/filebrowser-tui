//! File Browser TUI - Library
//!
//! A terminal-based file browser built with Rust and ratatui.
//! Cross-platform with Windows-specific optimizations (drive letters, UNC paths, etc.).

pub mod app;
pub mod commands;
pub mod file_ops;
pub mod state;
pub mod ui;

pub use state::App;
