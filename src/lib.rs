//============================================================================
// Copyright (C) 2026 Sunny Matato
//
// This program is free software. It comes without any warranty, to
// the extent permitted by applicable law. You can redistribute it
// and/or modify it under the terms of the Do What The Fuck You Want
// To Public License, Version 2, as published by Sam Hocevar.
// See http://www.wtfpl.net/ for more details.
//============================================================================
#![no_std]
#![allow(non_camel_case_types, non_snake_case)]

//============================================================================
mod sm_assert;
pub use sm_assert::SM_AssertHandler;
pub use sm_assert::SM_AssertInfo;
pub use sm_assert::SM_default_on_assert;
pub use sm_assert::SM_onAssert;
pub use sm_assert::SM_setOnAssert;
use sm_assert::{
    DBC_ASSERT, DBC_ENSURE, DBC_ERROR, DBC_INVARIANT, DBC_REQUIRE,

    SM_ASSERT_COLLECT_TO_TOP_DEPTH,SM_ASSERT_COLLECT_UNTIL_CURR_DEPTH,
    SM_ASSERT_INIT_TARGET_DESCENDANT,
    SM_ASSERT_NOT_INITIALIZED, SM_ASSERT_PATH_SLOT,
    SM_ASSERT_PUBLIC_TRANSITION_SOURCE, SM_ASSERT_TRANSITION_SOURCE,
};
//............................................................................
use core::ptr;

//============================================================================
// Trait contract
//============================================================================

/// HSM trait implemented by the user.
///
/// This is the Rust equivalent of the C-side naming contract that binds:
/// object type, event type, and top initial transition.
pub trait SM_HsmTrait: Sized + 'static {
    /// Concrete object type that owns the HSM instance and user data.
    type ActiveObject;
    /// Concrete event type dispatched to this HSM.
    type AO_Evt;

    /// Return the top-most initial transition target.
    fn TOP_initial(me: &mut Self::ActiveObject) -> SM_StatePtr<Self>;
}

//============================================================================
// State descriptor
//============================================================================

/// C-style `SM_StatePtr`: pointer to a static state descriptor.
///
/// `T` is constrained by `SM_HsmState<T>`, which requires `SM_HsmTrait`.
pub type SM_StatePtr<T> = &'static SM_HsmState<T>;

/// Entry/exit action function pointer.
pub type SM_ActionHandler<ActiveObject> = fn(me: &mut ActiveObject);

/// Initial transition function pointer.
pub type SM_InitHandler<T> = fn(
    me: &mut <T as SM_HsmTrait>::ActiveObject
) -> SM_StatePtr<T>;

/// State event handler function pointer.
pub type SM_StateHandler<T> = fn(
    me: &mut <T as SM_HsmTrait>::ActiveObject,
    e: &<T as SM_HsmTrait>::AO_Evt
) -> SM_RetState<T>;

/// Return value from a state handler.
#[derive(Clone, Copy)]
pub enum SM_RetState<T>
where
    T: SM_HsmTrait,
{
    /// Event handled and machine remains in current state.
    Handled,
    /// Bubble event handling to the parent state.
    Super,
    /// Transition to a new state pointer.
    Tran(SM_StatePtr<T>),
}

/// Static state descriptor.
///
/// This mirrors the C `SM_HsmState` table row: parent link, optional initial
/// transition, optional entry/exit actions, and the concrete state handler.
pub struct SM_HsmState<T>
where
    T: SM_HsmTrait,
{
    /// Parent state in the hierarchy. `None` means the top state.
    pub super_: Option<SM_StatePtr<T>>,
    /// Optional initial transition taken after entering this state.
    pub init_: Option<SM_InitHandler<T>>,
    /// Optional action called when entering this state.
    pub entry_: Option<SM_ActionHandler<<T as SM_HsmTrait>::ActiveObject>>,
    /// Optional action called when exiting this state.
    pub exit_: Option<SM_ActionHandler<<T as SM_HsmTrait>::ActiveObject>>,
    /// Handler that consumes events, bubbles to super, or requests transition.
    pub handler_: SM_StateHandler<T>,
}

//============================================================================
// Path operations
//============================================================================

struct SM_PathOps;

impl SM_PathOps {
    #[inline(always)]
    fn slot<T, const SM_MAX_NEST_DEPTH_: usize>(
        path: &[Option<SM_StatePtr<T>>; SM_MAX_NEST_DEPTH_],
        idx: usize,
    ) -> SM_StatePtr<T>
    where
        T: SM_HsmTrait,
    {
        match path[idx] {
            Some(state) => state,
            None => DBC_ASSERT!(SM_ASSERT_PATH_SLOT),
        }
    }

    fn collect_to_top<T, const SM_MAX_NEST_DEPTH_: usize>(
        target: SM_StatePtr<T>,
        path: &mut [Option<SM_StatePtr<T>>; SM_MAX_NEST_DEPTH_],
    ) -> usize
    where
        T: SM_HsmTrait,
    {
        let mut len = 0usize;
        let mut cursor = Some(target);

        while let Some(s) = cursor {
            DBC_INVARIANT!(SM_ASSERT_COLLECT_TO_TOP_DEPTH, len < SM_MAX_NEST_DEPTH_);
            path[len] = Some(s);
            len += 1;
            cursor = s.super_;
        }

        len
    }

    fn collect_until_curr<T, const SM_MAX_NEST_DEPTH_: usize>(
        curr: SM_StatePtr<T>,
        target: SM_StatePtr<T>,
        path: &mut [Option<SM_StatePtr<T>>; SM_MAX_NEST_DEPTH_],
    ) -> usize
    where
        T: SM_HsmTrait,
    {
        let mut len = 0usize;
        let mut cursor = Some(target);
        let mut reached_curr = false;

        while let Some(s) = cursor {
            if ptr::eq(s, curr) {
                reached_curr = true;
                break;
            }

            DBC_INVARIANT!(SM_ASSERT_COLLECT_UNTIL_CURR_DEPTH, len < SM_MAX_NEST_DEPTH_);
            path[len] = Some(s);
            len += 1;
            cursor = s.super_;
        }

        DBC_ENSURE!(SM_ASSERT_INIT_TARGET_DESCENDANT, reached_curr);

        len
    }
}

//============================================================================
// HSM engine
//============================================================================

/// Minimal embeddable hierarchical state machine container.
///
/// Keep one value as a plain field in user structs:
/// `sm_hsm_: SM_Hsm<MyTrait, 5>`.
pub struct SM_Hsm<T, const SM_MAX_NEST_DEPTH_: usize = 5>
where
    T: SM_HsmTrait,
{
    /// Current active state pointer.
    curr: Option<SM_StatePtr<T>>,
}

impl<T, const SM_MAX_NEST_DEPTH_: usize> Default for SM_Hsm<T, SM_MAX_NEST_DEPTH_>
where
    T: SM_HsmTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const SM_MAX_NEST_DEPTH_: usize> SM_Hsm<T, SM_MAX_NEST_DEPTH_>
where
    T: SM_HsmTrait,
{
    /// Create an empty HSM container. Call `init` before dispatching events.
    pub const fn new() -> Self {
        Self { curr: None }
    }

    /// Read current state (mostly for diagnostics/tests).
    pub fn curr(&self) -> Option<SM_StatePtr<T>> {
        self.curr
    }

    /// Initialize from `TOP_initial`, entering all ancestors then walking init links.
    pub fn init(&mut self, me: &mut T::ActiveObject) {
        let target = T::TOP_initial(me);
        let mut path = [None; SM_MAX_NEST_DEPTH_];
        let len = SM_PathOps::collect_to_top::<T, SM_MAX_NEST_DEPTH_>(target, &mut path);

        self.enter_primary_path(me, &path, len);
        self.curr = Some(target);
        self.follow_init_chain(me);
    }

    /// Dispatch an event through the active state chain.
    pub fn dispatch(&mut self, me: &mut T::ActiveObject, e: &T::AO_Evt) {
        let mut s = DBC_REQUIRE!(SM_ASSERT_NOT_INITIALIZED, self.curr);

        loop {
            match (s.handler_)(me, e) {
                SM_RetState::Handled => return,
                SM_RetState::Super => match s.super_ {
                    Some(super_) => s = super_,
                    None => return,
                },
                SM_RetState::Tran(target) => {
                    let curr = DBC_REQUIRE!(SM_ASSERT_NOT_INITIALIZED, self.curr);
                    self.transition_from_active(me, s, target, curr);
                    return;
                }
            }
        }
    }

    /// Execute a transition from `source` to `target`.
    pub fn transition(
        &mut self,
        me: &mut T::ActiveObject,
        source: SM_StatePtr<T>,
        target: SM_StatePtr<T>,
    ) {
        let curr = DBC_REQUIRE!(SM_ASSERT_NOT_INITIALIZED, self.curr);
        self.require_source_on_active_path(curr, source);
        self.transition_from_active(me, source, target, curr);
    }

    fn transition_from_active(
        &mut self,
        me: &mut T::ActiveObject,
        source: SM_StatePtr<T>,
        target: SM_StatePtr<T>,
        curr: SM_StatePtr<T>,
    ) {
        let mut path = [None; SM_MAX_NEST_DEPTH_];
        let len = SM_PathOps::collect_to_top::<T, SM_MAX_NEST_DEPTH_>(target, &mut path);

        let mut path_index = 0usize;
        let mut reached_source = false;
        let mut lca_found = false;
        let mut s = curr;

        loop {
            if ptr::eq(s, source) {
                reached_source = true;
            }

            if reached_source && !(ptr::eq(s, source) && ptr::eq(target, source)) {
                for idx in 0..len {
                    let ancestor = SM_PathOps::slot::<T, SM_MAX_NEST_DEPTH_>(&path, idx);

                    if ptr::eq(s, ancestor) {
                        lca_found = true;
                        path_index = idx;
                        break;
                    }
                }
            }

            if lca_found {
                break;
            }

            if let Some(exit_) = s.exit_ {
                exit_(me);
            }

            match s.super_ {
                Some(super_) => s = super_,
                None if reached_source => {
                    path_index = len;
                    break;
                }
                None => {
                    DBC_ERROR!(SM_ASSERT_TRANSITION_SOURCE);
                }
            }
        }

        while path_index > 0 {
            path_index -= 1;
            let s = SM_PathOps::slot::<T, SM_MAX_NEST_DEPTH_>(&path, path_index);
            if let Some(entry_) = s.entry_ {
                entry_(me);
            }
        }

        self.curr = Some(target);
        self.follow_init_chain(me);
    }

    fn require_source_on_active_path(&self, mut s: SM_StatePtr<T>, source: SM_StatePtr<T>) {
        loop {
            if ptr::eq(s, source) {
                return;
            }

            match s.super_ {
                Some(super_) => s = super_,
                None => DBC_ASSERT!(SM_ASSERT_PUBLIC_TRANSITION_SOURCE),
            }
        }
    }

    fn follow_init_chain(&mut self, me: &mut T::ActiveObject) {
        let mut path = [None; SM_MAX_NEST_DEPTH_];
        loop {
            let init_ = DBC_REQUIRE!(SM_ASSERT_NOT_INITIALIZED, self.curr).init_;

            let Some(init_) = init_ else {
                break;
            };
            let target = init_(me);
            let curr = DBC_REQUIRE!(SM_ASSERT_NOT_INITIALIZED, self.curr);
            let len =
                SM_PathOps::collect_until_curr::<T, SM_MAX_NEST_DEPTH_>(curr, target, &mut path);

            let mut idx = len;
            while idx > 0 {
                idx -= 1;
                let s = SM_PathOps::slot::<T, SM_MAX_NEST_DEPTH_>(&path, idx);
                self.curr = Some(s);
                if let Some(entry_) = s.entry_ {
                    entry_(me);
                }
            }
        }
    }

    fn enter_primary_path(
        &self,
        me: &mut T::ActiveObject,
        path: &[Option<SM_StatePtr<T>>; SM_MAX_NEST_DEPTH_],
        len: usize,
    ) {
        let mut idx = len;
        while idx > 0 {
            idx -= 1;
            let s = SM_PathOps::slot::<T, SM_MAX_NEST_DEPTH_>(path, idx);
            if let Some(entry_) = s.entry_ {
                entry_(me);
            }
        }
    }
}
