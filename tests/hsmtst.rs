#[path = "../examples/hsmtst_fixture/mod.rs"]
mod hsmtst_fixture;

use hsmtst_fixture::{
    SmHsmTst, SmHsmTstSig, SmHsmTst_run_sequence, SMHSMTST_SEQUENCE_A, SMHSMTST_SEQUENCE_B,
};
use rs_hsm_::SM_DispResult;

#[test]
fn startup_trace_matches_reference() {
    let run = SmHsmTst_run_sequence(&[]);

    assert_eq!(
        run.trace,
        "top-INIT.s-ENTRY.s2-ENTRY.s2-INIT.s21-ENTRY.s211-ENTRY."
    );
    assert_eq!(run.curr_name, "S211");
}

#[test]
fn hsmtst_sequence_b_matches_reference_output() {
    let run = SmHsmTst_run_sequence(SMHSMTST_SEQUENCE_B);

    assert_eq!(
        run.trace,
        concat!(
            "top-INIT.s-ENTRY.s2-ENTRY.s2-INIT.s21-ENTRY.s211-ENTRY.",
            "\n",
            "s21-G.s211-EXIT.s21-EXIT.s2-EXIT.s1-ENTRY.s1-INIT.s11-ENTRY.",
            "\n",
            "s1-I.",
            "\n",
            "s1-A.s11-EXIT.s1-EXIT.s1-ENTRY.s1-INIT.s11-ENTRY.",
            "\n",
            "s1-D.s11-EXIT.s1-EXIT.s-INIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s11-D.s11-EXIT.s1-INIT.s11-ENTRY.",
            "\n",
            "s1-C.s11-EXIT.s1-EXIT.s2-ENTRY.s2-INIT.s21-ENTRY.s211-ENTRY.",
            "\n",
            "s-E.s211-EXIT.s21-EXIT.s2-EXIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s-E.s11-EXIT.s1-EXIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s11-G.s11-EXIT.s1-EXIT.s2-ENTRY.s21-ENTRY.s211-ENTRY.",
            "\n",
            "s2-I.",
            "\n",
            "s-I."
        )
    );
    assert_eq!(run.curr_name, "S211");
}

#[test]
fn hsmtst_sequence_a_matches_reference_output() {
    let run = SmHsmTst_run_sequence(SMHSMTST_SEQUENCE_A);

    assert_eq!(
        run.trace,
        concat!(
            "top-INIT.s-ENTRY.s2-ENTRY.s2-INIT.s21-ENTRY.s211-ENTRY.",
            "\n",
            "s21-A.s211-EXIT.s21-EXIT.s21-ENTRY.s21-INIT.s211-ENTRY.",
            "\n",
            "s21-B.s211-EXIT.s211-ENTRY.",
            "\n",
            "s211-D.s211-EXIT.s21-INIT.s211-ENTRY.",
            "\n",
            "s-E.s211-EXIT.s21-EXIT.s2-EXIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s1-I.",
            "\n",
            "s1-F.s11-EXIT.s1-EXIT.s2-ENTRY.s21-ENTRY.s211-ENTRY.",
            "\n",
            "s2-I.",
            "\n",
            "s-I.",
            "\n",
            "s2-F.s211-EXIT.s21-EXIT.s2-EXIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s1-A.s11-EXIT.s1-EXIT.s1-ENTRY.s1-INIT.s11-ENTRY.",
            "\n",
            "s1-B.s11-EXIT.s11-ENTRY.",
            "\n",
            "s1-D.s11-EXIT.s1-EXIT.s-INIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s11-D.s11-EXIT.s1-INIT.s11-ENTRY.",
            "\n",
            "s-E.s11-EXIT.s1-EXIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s11-G.s11-EXIT.s1-EXIT.s2-ENTRY.s21-ENTRY.s211-ENTRY.",
            "\n",
            "s211-H.s211-EXIT.s21-EXIT.s2-EXIT.s-INIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s11-H.s11-EXIT.s1-EXIT.s-INIT.s1-ENTRY.s11-ENTRY.",
            "\n",
            "s1-C.s11-EXIT.s1-EXIT.s2-ENTRY.s2-INIT.s21-ENTRY.s211-ENTRY.",
            "\n",
            "s21-G.s211-EXIT.s21-EXIT.s2-EXIT.s1-ENTRY.s1-INIT.s11-ENTRY.",
            "\n",
            "s1-C.s11-EXIT.s1-EXIT.s2-ENTRY.s2-INIT.s21-ENTRY.s211-ENTRY.",
            "\n",
            "s2-C.s211-EXIT.s21-EXIT.s2-EXIT.s1-ENTRY.s1-INIT.s11-ENTRY."
        )
    );
    assert_eq!(run.curr_name, "S11");
}

#[test]
fn ignored_event_keeps_state_and_reports_ignored() {
    let mut machine = SmHsmTst::new();
    let trace_before = machine.trace().to_string();

    let outcome = machine.dispatch(SmHsmTstSig::Z_SIG);

    assert_eq!(outcome, SM_DispResult::Ignored);
    assert_eq!(machine.trace(), trace_before);
    assert_eq!(machine.curr_name(), "S211");
}

#[test]
#[should_panic(expected = "initial transition target must be a descendant of current state")]
fn init_target_must_be_descendant_of_current_state() {
    struct BadCtx;
    struct BadChart;

    impl rs_hsm_::SM_HsmImpl for BadChart {
        type Context = BadCtx;
        type Event = ();

        fn TOP_initial(_me: &mut Self::Context) -> rs_hsm_::SM_StatePtr<Self> {
            &BAD_PARENT
        }
    }

    fn bad_parent_init(_me: &mut BadCtx) -> rs_hsm_::SM_StatePtr<BadChart> {
        &BAD_SIBLING
    }

    fn bad_state_handler(_me: &mut BadCtx, _e: &()) -> rs_hsm_::SM_RetState<BadChart> {
        rs_hsm_::SM_RetState::Handled
    }

    static BAD_PARENT: rs_hsm_::SM_HsmState<BadChart> = rs_hsm_::SM_HsmState {
        super_: None,
        init_: Some(bad_parent_init),
        entry_: None,
        exit_: None,
        handler_: bad_state_handler,
    };

    static BAD_SIBLING: rs_hsm_::SM_HsmState<BadChart> = rs_hsm_::SM_HsmState {
        super_: None,
        init_: None,
        entry_: None,
        exit_: None,
        handler_: bad_state_handler,
    };

    let mut ctx = BadCtx;
    let mut hsm = rs_hsm_::SM_Hsm::<BadChart>::new();
    hsm.init(&mut ctx);
}
