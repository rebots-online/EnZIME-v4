#![forbid(unsafe_code)]

//! Ring-1 application services: complete use cases and orchestration.
//!
//! Each module owns one use-case family and depends only on `domain` and
//! `ports` — never on a concrete provider crate (A-02).

pub mod activation;
pub mod annotation_assets;
pub mod ask;
pub mod bootstrap;
pub mod home;
pub mod install;
pub mod library;
pub mod model;
pub mod readiness;
pub mod reader_nav;
pub mod sidecar;
pub mod transfer;
