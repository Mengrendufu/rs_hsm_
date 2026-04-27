//============================================================================
// Copyright (C) 2026 Sunny Matato
//
// This program is free software. It comes without any warranty, to
// the extent permitted by applicable law. You can redistribute it
// and/or modify it under the terms of the Do What The Fuck You Want
// To Public License, Version 2, as published by Sam Hocevar.
// See http://www.wtfpl.net/ for more details.
//============================================================================
//! Contract assertions used by the HSM runtime.
//!
//! The implementation is intentionally small and `core`-only so it can be used in
//! embedded/no_std environments without allocation.

macro_rules! DBC_ASSERT {
    ($label:expr) => {{
        $crate::SM_onAssert($crate::SM_AssertInfo {
            module: module_path!(),
            label: $label,
        })
    }};
}

macro_rules! DBC_REQUIRE {
    ($label:expr, $value:expr) => {{
        match $value {
            Some(value) => value,
            None => DBC_ASSERT!($label),
        }
    }};
}

macro_rules! DBC_ENSURE {
    ($label:expr, $expr:expr) => {{
        if !($expr) {
            DBC_ASSERT!($label);
        }
    }};
}

macro_rules! DBC_INVARIANT {
    ($label:expr, $expr:expr) => {{
        if !($expr) {
            DBC_ASSERT!($label);
        }
    }};
}

macro_rules! DBC_ERROR {
    ($label:expr) => {{
        DBC_ASSERT!($label);
    }};
}

pub(crate) use DBC_ASSERT;
pub(crate) use DBC_ENSURE;
pub(crate) use DBC_ERROR;
pub(crate) use DBC_INVARIANT;
pub(crate) use DBC_REQUIRE;

pub(crate) const SM_ASSERT_PATH_SLOT: u32 = 100;
pub(crate) const SM_ASSERT_COLLECT_TO_TOP_DEPTH: u32 = 110;
pub(crate) const SM_ASSERT_COLLECT_UNTIL_CURR_DEPTH: u32 = 120;
pub(crate) const SM_ASSERT_INIT_TARGET_DESCENDANT: u32 = 130;
pub(crate) const SM_ASSERT_NOT_INITIALIZED: u32 = 200;
pub(crate) const SM_ASSERT_TRANSITION_SOURCE: u32 = 300;
pub(crate) const SM_ASSERT_PUBLIC_TRANSITION_SOURCE: u32 = 310;

/// Compact payload forwarded to a customized assert handler.
#[derive(Clone, Copy, Debug)]
pub struct SM_AssertInfo {
    /// Module where the assertion failed.
    pub module: &'static str,
    /// Stable assertion label, unique within the module.
    pub label: u32,
}

/// System-level assert hook signature.
pub type SM_AssertHandler = fn(info: SM_AssertInfo) -> !;

static mut SM_ASSERT_HANDLER: SM_AssertHandler = SM_default_on_assert;

impl SM_AssertInfo {
    #[inline(always)]
    pub const fn new(module: &'static str, label: u32) -> Self {
        Self { module, label }
    }
}

/// Install the system-level assert hook.
///
/// # Safety
///
/// Call this during single-threaded board/application startup, before any HSM
/// can dispatch events or assert from another context. Changing the hook while
/// another context can call `SM_onAssert` is a data race.
#[inline(always)]
pub unsafe fn SM_setOnAssert(handler: SM_AssertHandler) {
    unsafe {
        SM_ASSERT_HANDLER = handler;
    }
}

/// System-level assert entry used by all DBC macros.
#[cold]
#[inline(never)]
pub fn SM_onAssert(info: SM_AssertInfo) -> ! {
    let handler = unsafe { SM_ASSERT_HANDLER };
    handler(info)
}

/// Default fail-fast implementation used by embedded/state-chart hosts that do not
/// provide their own assert policy.
///
/// The default behavior keeps the existing panic-based behavior for compatibility.
#[cold]
#[inline(never)]
pub fn SM_default_on_assert(info: SM_AssertInfo) -> ! {
    panic!("DBC assertion failed: {}:{}", info.module, info.label);
}
