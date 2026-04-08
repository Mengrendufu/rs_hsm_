#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ptr;
use rs_hsm_::{SM_DispResult, SM_Hsm, SM_HsmImpl, SM_HsmState, SM_RetState, SM_StatePtr};
use std::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmHsmTstSig {
    A_SIG,
    B_SIG,
    C_SIG,
    D_SIG,
    E_SIG,
    F_SIG,
    G_SIG,
    H_SIG,
    I_SIG,
    #[allow(dead_code)]
    Z_SIG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmHsmTstEvt {
    pub sig: SmHsmTstSig,
}

impl SmHsmTstEvt {
    pub const fn new(sig: SmHsmTstSig) -> Self {
        Self { sig }
    }
}

pub const SMHSMTST_SEQUENCE_A: &[SmHsmTstSig] = &[
    SmHsmTstSig::A_SIG,
    SmHsmTstSig::B_SIG,
    SmHsmTstSig::D_SIG,
    SmHsmTstSig::E_SIG,
    SmHsmTstSig::I_SIG,
    SmHsmTstSig::F_SIG,
    SmHsmTstSig::I_SIG,
    SmHsmTstSig::I_SIG,
    SmHsmTstSig::F_SIG,
    SmHsmTstSig::A_SIG,
    SmHsmTstSig::B_SIG,
    SmHsmTstSig::D_SIG,
    SmHsmTstSig::D_SIG,
    SmHsmTstSig::E_SIG,
    SmHsmTstSig::G_SIG,
    SmHsmTstSig::H_SIG,
    SmHsmTstSig::H_SIG,
    SmHsmTstSig::C_SIG,
    SmHsmTstSig::G_SIG,
    SmHsmTstSig::C_SIG,
    SmHsmTstSig::C_SIG,
];

pub const SMHSMTST_SEQUENCE_B: &[SmHsmTstSig] = &[
    SmHsmTstSig::G_SIG,
    SmHsmTstSig::I_SIG,
    SmHsmTstSig::A_SIG,
    SmHsmTstSig::D_SIG,
    SmHsmTstSig::D_SIG,
    SmHsmTstSig::C_SIG,
    SmHsmTstSig::E_SIG,
    SmHsmTstSig::E_SIG,
    SmHsmTstSig::G_SIG,
    SmHsmTstSig::I_SIG,
    SmHsmTstSig::I_SIG,
];

#[derive(Default)]
struct SmHsmTstCtx {
    trace: String,
    foo: u8,
}

struct SmHsmTstStateChart;

impl SM_HsmImpl for SmHsmTstStateChart {
    type Context = SmHsmTstCtx;
    type Event = SmHsmTstEvt;

    fn TOP_initial(me: &mut Self::Context) -> SM_StatePtr<Self> {
        SmHsmTst_trace(me, "top-INIT.");
        &SmHsmTst_s2
    }
}

fn SmHsmTst_trace(me: &mut SmHsmTstCtx, msg: &str) {
    me.trace.push_str(msg);
}

fn SmHsmTst_s_init_(me: &mut SmHsmTstCtx) -> SM_StatePtr<SmHsmTstStateChart> {
    SmHsmTst_trace(me, "s-INIT.");
    &SmHsmTst_s11
}

fn SmHsmTst_s_entry_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s-ENTRY.");
}

fn SmHsmTst_s_exit_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s-EXIT.");
}

fn SmHsmTst_s_(me: &mut SmHsmTstCtx, e: &SmHsmTstEvt) -> SM_RetState<SmHsmTstStateChart> {
    match e.sig {
        SmHsmTstSig::I_SIG if me.foo != 0 => {
            me.foo = 0;
            SmHsmTst_trace(me, "s-I.");
            SM_RetState::Handled
        }
        SmHsmTstSig::I_SIG => SM_RetState::Super,
        SmHsmTstSig::E_SIG => {
            SmHsmTst_trace(me, "s-E.");
            SM_RetState::Tran(&SmHsmTst_s11)
        }
        _ => SM_RetState::Super,
    }
}

fn SmHsmTst_s1_init_(me: &mut SmHsmTstCtx) -> SM_StatePtr<SmHsmTstStateChart> {
    SmHsmTst_trace(me, "s1-INIT.");
    &SmHsmTst_s11
}

fn SmHsmTst_s1_entry_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s1-ENTRY.");
}

fn SmHsmTst_s1_exit_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s1-EXIT.");
}

fn SmHsmTst_s1_(me: &mut SmHsmTstCtx, e: &SmHsmTstEvt) -> SM_RetState<SmHsmTstStateChart> {
    match e.sig {
        SmHsmTstSig::I_SIG => {
            SmHsmTst_trace(me, "s1-I.");
            SM_RetState::Handled
        }
        SmHsmTstSig::B_SIG => {
            SmHsmTst_trace(me, "s1-B.");
            SM_RetState::Tran(&SmHsmTst_s11)
        }
        SmHsmTstSig::A_SIG => {
            SmHsmTst_trace(me, "s1-A.");
            SM_RetState::Tran(&SmHsmTst_s1)
        }
        SmHsmTstSig::F_SIG => {
            SmHsmTst_trace(me, "s1-F.");
            SM_RetState::Tran(&SmHsmTst_s211)
        }
        SmHsmTstSig::C_SIG => {
            SmHsmTst_trace(me, "s1-C.");
            SM_RetState::Tran(&SmHsmTst_s2)
        }
        SmHsmTstSig::D_SIG if me.foo == 0 => {
            me.foo = 1;
            SmHsmTst_trace(me, "s1-D.");
            SM_RetState::Tran(&SmHsmTst_s)
        }
        SmHsmTstSig::D_SIG => SM_RetState::Super,
        _ => SM_RetState::Super,
    }
}

fn SmHsmTst_s11_entry_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s11-ENTRY.");
}

fn SmHsmTst_s11_exit_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s11-EXIT.");
}

fn SmHsmTst_s11_(me: &mut SmHsmTstCtx, e: &SmHsmTstEvt) -> SM_RetState<SmHsmTstStateChart> {
    match e.sig {
        SmHsmTstSig::H_SIG => {
            SmHsmTst_trace(me, "s11-H.");
            SM_RetState::Tran(&SmHsmTst_s)
        }
        SmHsmTstSig::D_SIG if me.foo != 0 => {
            me.foo = 0;
            SmHsmTst_trace(me, "s11-D.");
            SM_RetState::Tran(&SmHsmTst_s1)
        }
        SmHsmTstSig::D_SIG => SM_RetState::Super,
        SmHsmTstSig::G_SIG => {
            SmHsmTst_trace(me, "s11-G.");
            SM_RetState::Tran(&SmHsmTst_s211)
        }
        _ => SM_RetState::Super,
    }
}

fn SmHsmTst_s2_init_(me: &mut SmHsmTstCtx) -> SM_StatePtr<SmHsmTstStateChart> {
    SmHsmTst_trace(me, "s2-INIT.");
    &SmHsmTst_s211
}

fn SmHsmTst_s2_entry_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s2-ENTRY.");
}

fn SmHsmTst_s2_exit_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s2-EXIT.");
}

fn SmHsmTst_s2_(me: &mut SmHsmTstCtx, e: &SmHsmTstEvt) -> SM_RetState<SmHsmTstStateChart> {
    match e.sig {
        SmHsmTstSig::I_SIG if me.foo == 0 => {
            me.foo = 1;
            SmHsmTst_trace(me, "s2-I.");
            SM_RetState::Handled
        }
        SmHsmTstSig::I_SIG => SM_RetState::Super,
        SmHsmTstSig::C_SIG => {
            SmHsmTst_trace(me, "s2-C.");
            SM_RetState::Tran(&SmHsmTst_s1)
        }
        SmHsmTstSig::F_SIG => {
            SmHsmTst_trace(me, "s2-F.");
            SM_RetState::Tran(&SmHsmTst_s11)
        }
        _ => SM_RetState::Super,
    }
}

fn SmHsmTst_s21_init_(me: &mut SmHsmTstCtx) -> SM_StatePtr<SmHsmTstStateChart> {
    SmHsmTst_trace(me, "s21-INIT.");
    &SmHsmTst_s211
}

fn SmHsmTst_s21_entry_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s21-ENTRY.");
}

fn SmHsmTst_s21_exit_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s21-EXIT.");
}

fn SmHsmTst_s21_(me: &mut SmHsmTstCtx, e: &SmHsmTstEvt) -> SM_RetState<SmHsmTstStateChart> {
    match e.sig {
        SmHsmTstSig::G_SIG => {
            SmHsmTst_trace(me, "s21-G.");
            SM_RetState::Tran(&SmHsmTst_s1)
        }
        SmHsmTstSig::A_SIG => {
            SmHsmTst_trace(me, "s21-A.");
            SM_RetState::Tran(&SmHsmTst_s21)
        }
        SmHsmTstSig::B_SIG => {
            SmHsmTst_trace(me, "s21-B.");
            SM_RetState::Tran(&SmHsmTst_s211)
        }
        _ => SM_RetState::Super,
    }
}

fn SmHsmTst_s211_entry_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s211-ENTRY.");
}

fn SmHsmTst_s211_exit_(me: &mut SmHsmTstCtx) {
    SmHsmTst_trace(me, "s211-EXIT.");
}

fn SmHsmTst_s211_(me: &mut SmHsmTstCtx, e: &SmHsmTstEvt) -> SM_RetState<SmHsmTstStateChart> {
    match e.sig {
        SmHsmTstSig::H_SIG => {
            SmHsmTst_trace(me, "s211-H.");
            SM_RetState::Tran(&SmHsmTst_s)
        }
        SmHsmTstSig::D_SIG => {
            SmHsmTst_trace(me, "s211-D.");
            SM_RetState::Tran(&SmHsmTst_s21)
        }
        _ => SM_RetState::Super,
    }
}

static SmHsmTst_s: SM_HsmState<SmHsmTstStateChart> = SM_HsmState {
    super_: None,
    init_: Some(SmHsmTst_s_init_),
    entry_: Some(SmHsmTst_s_entry_),
    exit_: Some(SmHsmTst_s_exit_),
    handler_: SmHsmTst_s_,
};

static SmHsmTst_s1: SM_HsmState<SmHsmTstStateChart> = SM_HsmState {
    super_: Some(&SmHsmTst_s),
    init_: Some(SmHsmTst_s1_init_),
    entry_: Some(SmHsmTst_s1_entry_),
    exit_: Some(SmHsmTst_s1_exit_),
    handler_: SmHsmTst_s1_,
};

static SmHsmTst_s11: SM_HsmState<SmHsmTstStateChart> = SM_HsmState {
    super_: Some(&SmHsmTst_s1),
    init_: None,
    entry_: Some(SmHsmTst_s11_entry_),
    exit_: Some(SmHsmTst_s11_exit_),
    handler_: SmHsmTst_s11_,
};

static SmHsmTst_s2: SM_HsmState<SmHsmTstStateChart> = SM_HsmState {
    super_: Some(&SmHsmTst_s),
    init_: Some(SmHsmTst_s2_init_),
    entry_: Some(SmHsmTst_s2_entry_),
    exit_: Some(SmHsmTst_s2_exit_),
    handler_: SmHsmTst_s2_,
};

static SmHsmTst_s21: SM_HsmState<SmHsmTstStateChart> = SM_HsmState {
    super_: Some(&SmHsmTst_s2),
    init_: Some(SmHsmTst_s21_init_),
    entry_: Some(SmHsmTst_s21_entry_),
    exit_: Some(SmHsmTst_s21_exit_),
    handler_: SmHsmTst_s21_,
};

static SmHsmTst_s211: SM_HsmState<SmHsmTstStateChart> = SM_HsmState {
    super_: Some(&SmHsmTst_s21),
    init_: None,
    entry_: Some(SmHsmTst_s211_entry_),
    exit_: Some(SmHsmTst_s211_exit_),
    handler_: SmHsmTst_s211_,
};

fn SmHsmTst_state_name(state: SM_StatePtr<SmHsmTstStateChart>) -> &'static str {
    if ptr::eq(state, &SmHsmTst_s) {
        "S"
    } else if ptr::eq(state, &SmHsmTst_s1) {
        "S1"
    } else if ptr::eq(state, &SmHsmTst_s11) {
        "S11"
    } else if ptr::eq(state, &SmHsmTst_s2) {
        "S2"
    } else if ptr::eq(state, &SmHsmTst_s21) {
        "S21"
    } else if ptr::eq(state, &SmHsmTst_s211) {
        "S211"
    } else {
        panic!("unknown hsmtst state")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SmHsmTstRun {
    pub trace: String,
    pub curr_name: &'static str,
}

pub struct SmHsmTst {
    me: SmHsmTstCtx,
    sm_hsm_: SM_Hsm<SmHsmTstStateChart, 5>,
}

impl SmHsmTst {
    pub fn new() -> Self {
        let mut me = SmHsmTstCtx::default();
        let mut sm_hsm_ = SM_Hsm::<SmHsmTstStateChart, 5>::new();
        sm_hsm_.init(&mut me);
        Self { me, sm_hsm_ }
    }

    #[allow(dead_code)]
    pub fn trace(&self) -> &str {
        &self.me.trace
    }

    #[allow(dead_code)]
    pub fn curr_name(&self) -> &'static str {
        SmHsmTst_state_name(
            self.sm_hsm_
                .curr()
                .expect("hsmtst machine must stay initialized"),
        )
    }

    pub fn dispatch(&mut self, sig: SmHsmTstSig) -> SM_DispResult {
        let e = SmHsmTstEvt::new(sig);
        self.sm_hsm_.dispatch(&mut self.me, &e)
    }

    pub fn dispatch_with_separator(&mut self, sig: SmHsmTstSig) -> SM_DispResult {
        SmHsmTst_trace(&mut self.me, "\n");
        self.dispatch(sig)
    }

    pub fn finish(self) -> SmHsmTstRun {
        let curr_name = SmHsmTst_state_name(
            self.sm_hsm_
                .curr()
                .expect("hsmtst machine must stay initialized"),
        );
        let trace = self.me.trace;
        SmHsmTstRun { trace, curr_name }
    }
}

impl Default for SmHsmTst {
    fn default() -> Self {
        Self::new()
    }
}

pub fn SmHsmTst_run_sequence(signals: &[SmHsmTstSig]) -> SmHsmTstRun {
    let mut machine = SmHsmTst::new();
    for &sig in signals {
        machine.dispatch_with_separator(sig);
    }
    machine.finish()
}

#[allow(dead_code)]
pub fn SmHsmTst_select_sequence(arg: Option<&str>) -> &'static [SmHsmTstSig] {
    match arg.unwrap_or("b") {
        "startup" => &[],
        "a" | "A" => SMHSMTST_SEQUENCE_A,
        "b" | "B" => SMHSMTST_SEQUENCE_B,
        other => panic!("unknown sequence: {other}. use: startup | a | b"),
    }
}
