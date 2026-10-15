#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! `craft-scaffold` library: validated [`plan`], pure [`render`], disk [`write`].
//! The binary (`src/main.rs`) adds prompts, flags and process exit.

pub mod plan;
pub mod render;
pub mod write;
