//! File Browser TUI - Library
//!
//! A terminal-based file browser for Windows built with Rust and ratatui.

pub mod app;
pub mod commands;
pub mod file_ops;
pub mod state;
pub mod ui;

pub use state::App;
