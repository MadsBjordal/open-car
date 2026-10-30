// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared core of open-car.
//!
//! open-car does not implement Android Auto or CarPlay itself: the projection engine is LIVI
//! (ADR-0002). This crate holds what open-car adds around it: hardware profiles and the
//! compatibility database, hardware probing, readiness checks, LIVI configuration and diagnostics.

pub mod compat;
pub mod config;
pub mod diag;
pub mod fsutil;
pub mod livi;
pub mod probe;
pub mod profile;
pub mod readiness;

/// The open-car version, from Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
