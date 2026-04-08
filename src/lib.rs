#![no_std]
#![allow(non_camel_case_types, non_snake_case)]

use core::ptr;

pub type SM_ActionHandler<C> = fn(&mut C);
pub type SM_StatePtr<C> = &'static SM_HsmState<C>;
pub type SM_InitHandler<C> = fn(&mut <C as SM_HsmImpl>::Context) -> SM_StatePtr<C>;
pub type SM_StateHandler<C> =
    fn(&mut <C as SM_HsmImpl>::Context, &<C as SM_HsmImpl>::Event) -> SM_RetState<C>;

#[derive(Clone, Copy)]
pub enum SM_RetState<C: SM_HsmImpl> {
    Handled,
    Super,
    Tran(SM_StatePtr<C>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SM_DispResult {
    Handled,
    Tran,
    Ignored,
}

pub struct SM_HsmState<C: SM_HsmImpl> {
    pub super_: Option<SM_StatePtr<C>>,
    pub init_: Option<SM_InitHandler<C>>,
    pub entry_: Option<SM_ActionHandler<C::Context>>,
    pub exit_: Option<SM_ActionHandler<C::Context>>,
    pub handler_: SM_StateHandler<C>,
}

pub trait SM_HsmImpl: Sized + 'static {
    type Context;
    type Event;

    fn TOP_initial(me: &mut Self::Context) -> SM_StatePtr<Self>;
}

pub struct SM_Hsm<C: SM_HsmImpl, const SM_MAX_NEST_DEPTH_: usize = 8> {
    curr: Option<SM_StatePtr<C>>,
}

impl<C: SM_HsmImpl, const SM_MAX_NEST_DEPTH_: usize> Default for SM_Hsm<C, SM_MAX_NEST_DEPTH_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: SM_HsmImpl, const SM_MAX_NEST_DEPTH_: usize> SM_Hsm<C, SM_MAX_NEST_DEPTH_> {
    pub const fn new() -> Self {
        Self { curr: None }
    }

    pub fn curr(&self) -> Option<SM_StatePtr<C>> {
        self.curr
    }

    pub fn init(&mut self, me: &mut C::Context) {
        let target = C::TOP_initial(me);
        let mut path = [None; SM_MAX_NEST_DEPTH_];
        let len = self.collect_path_to_top(target, &mut path);

        self.enter_primary_path(me, &path, len);
        self.curr = Some(target);
        self.follow_init_chain(me);
    }

    pub fn dispatch(&mut self, me: &mut C::Context, e: &C::Event) -> SM_DispResult {
        let mut s = self.curr.expect("HSM must be initialized before dispatch");

        loop {
            match (s.handler_)(me, e) {
                SM_RetState::Handled => return SM_DispResult::Handled,
                SM_RetState::Super => match s.super_ {
                    Some(super_) => s = super_,
                    None => return SM_DispResult::Ignored,
                },
                SM_RetState::Tran(target) => {
                    self.transition(me, s, target);
                    return SM_DispResult::Tran;
                }
            }
        }
    }

    pub fn transition(&mut self, me: &mut C::Context, source: SM_StatePtr<C>, target: SM_StatePtr<C>) {
        let mut path = [None; SM_MAX_NEST_DEPTH_];
        let len = self.collect_path_to_top(target, &mut path);

        let mut path_index = 0usize;
        let mut reached_source = false;
        let mut lca_found = false;
        let mut s = self.curr.expect("HSM must be initialized before transition");

        loop {
            if ptr::eq(s, source) {
                reached_source = true;
            }

            if reached_source && !(ptr::eq(s, source) && ptr::eq(target, source)) {
                for idx in 0..len {
                    if ptr::eq(s, path[idx].expect("path slot must be initialized")) {
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
                None => panic!("transition source is not on the current active path"),
            }
        }

        while path_index > 0 {
            path_index -= 1;
            let s = path[path_index].expect("path slot must be initialized");
            if let Some(entry_) = s.entry_ {
                entry_(me);
            }
        }

        self.curr = Some(target);
        self.follow_init_chain(me);
    }

    fn follow_init_chain(&mut self, me: &mut C::Context) {
        let mut path = [None; SM_MAX_NEST_DEPTH_];

        while let Some(init_) = self.curr.expect("current state must exist").init_ {
            let target = init_(me);
            let len = self.collect_path_until_curr(target, &mut path);

            let mut idx = len;
            while idx > 0 {
                idx -= 1;
                let s = path[idx].expect("path slot must be initialized");
                self.curr = Some(s);
                if let Some(entry_) = s.entry_ {
                    entry_(me);
                }
            }
        }
    }

    fn collect_path_to_top(
        &self,
        target: SM_StatePtr<C>,
        path: &mut [Option<SM_StatePtr<C>>; SM_MAX_NEST_DEPTH_],
    ) -> usize {
        let mut len = 0usize;
        let mut cursor = Some(target);

        while let Some(s) = cursor {
            assert!(
                len < SM_MAX_NEST_DEPTH_,
                "state nesting exceeds SM_MAX_NEST_DEPTH_"
            );
            path[len] = Some(s);
            len += 1;
            cursor = s.super_;
        }

        len
    }

    fn collect_path_until_curr(
        &self,
        target: SM_StatePtr<C>,
        path: &mut [Option<SM_StatePtr<C>>; SM_MAX_NEST_DEPTH_],
    ) -> usize {
        let mut len = 0usize;
        let stop = self.curr;
        let mut cursor = Some(target);

        while let Some(s) = cursor {
            match stop {
                Some(curr) if ptr::eq(s, curr) => break,
                _ => {}
            }
            assert!(
                len < SM_MAX_NEST_DEPTH_,
                "state nesting exceeds SM_MAX_NEST_DEPTH_"
            );
            path[len] = Some(s);
            len += 1;
            cursor = s.super_;
        }

        len
    }

    fn enter_primary_path(
        &self,
        me: &mut C::Context,
        path: &[Option<SM_StatePtr<C>>; SM_MAX_NEST_DEPTH_],
        len: usize,
    ) {
        let mut idx = len;
        while idx > 0 {
            idx -= 1;
            let s = path[idx].expect("path slot must be initialized");
            if let Some(entry_) = s.entry_ {
                entry_(me);
            }
        }
    }
}
