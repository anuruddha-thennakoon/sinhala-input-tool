//! Windows text service (TSF) for Sinhala input. See PLAN.md, Phase 5.
//!
//! `keys` is platform-neutral and unit-tested everywhere; everything else only
//! exists on Windows.

pub mod keys;

#[cfg(windows)]
mod dll;
#[cfg(windows)]
mod guids;
#[cfg(windows)]
mod log;
#[cfg(windows)]
mod popup;
#[cfg(windows)]
mod registration;
#[cfg(windows)]
mod service;
