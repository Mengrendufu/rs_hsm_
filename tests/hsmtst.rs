#[path = "../examples/hsmtst_fixture/mod.rs"]
mod hsmtst_fixture;

use hsmtst_fixture::{
    SMHSMTST_SEQUENCE_A, SMHSMTST_SEQUENCE_B, SmHsmTst, SmHsmTst_run_sequence, SmHsmTstSig,
};
use std::sync::Once;

fn test_on_assert(info: rs_hsm_::SM_AssertInfo) -> ! {
    panic!("DBC assertion failed: {}:{}", info.module, info.label);
}

fn install_test_assert_handler() {
    static INSTALL: Once = Once::new();

    INSTALL.call_once(|| unsafe {
        rs_hsm_::SM_setOnAssert(test_on_assert);
    });
}

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
fn ignored_event_keeps_state() {
    let mut machine = SmHsmTst::new();
    let trace_before = machine.trace().to_string();

    machine.dispatch(SmHsmTstSig::Z_SIG);

    assert_eq!(machine.trace(), trace_before);
    assert_eq!(machine.curr_name(), "S211");
}

#[test]
#[should_panic(expected = "DBC assertion failed: rs_hsm_:130")]
fn init_target_must_be_descendant_of_current_state() {
    install_test_assert_handler();

    struct BadObject;
    struct BadSpec;

    impl rs_hsm_::SM_HsmTrait for BadSpec {
        type ActiveObject = BadObject;
        type AO_Evt = ();

        fn TOP_initial(_me: &mut Self::ActiveObject) -> rs_hsm_::SM_StatePtr<Self> {
            &BAD_PARENT
        }
    }

    fn bad_parent_init(_me: &mut BadObject) -> rs_hsm_::SM_StatePtr<BadSpec> {
        &BAD_SIBLING
    }

    fn bad_state_handler(_me: &mut BadObject, _e: &()) -> rs_hsm_::SM_RetState<BadSpec> {
        rs_hsm_::SM_RetState::Handled
    }

    static BAD_PARENT: rs_hsm_::SM_HsmState<BadSpec> = rs_hsm_::SM_HsmState {
        super_: None,
        init_: Some(bad_parent_init),
        entry_: None,
        exit_: None,
        handler_: bad_state_handler,
    };

    static BAD_SIBLING: rs_hsm_::SM_HsmState<BadSpec> = rs_hsm_::SM_HsmState {
        super_: None,
        init_: None,
        entry_: None,
        exit_: None,
        handler_: bad_state_handler,
    };

    let mut ctx = BadObject;
    let mut hsm = rs_hsm_::SM_Hsm::<BadSpec>::new();
    hsm.init(&mut ctx);
}

#[test]
fn invalid_transition_source_fails_before_exit_action() {
    install_test_assert_handler();

    struct BadObject {
        exited: bool,
    }
    struct BadSpec;

    impl rs_hsm_::SM_HsmTrait for BadSpec {
        type ActiveObject = BadObject;
        type AO_Evt = ();

        fn TOP_initial(_me: &mut Self::ActiveObject) -> rs_hsm_::SM_StatePtr<Self> {
            &BAD_CHILD
        }
    }

    fn bad_child_exit(me: &mut BadObject) {
        me.exited = true;
    }

    fn bad_state_handler(_me: &mut BadObject, _e: &()) -> rs_hsm_::SM_RetState<BadSpec> {
        rs_hsm_::SM_RetState::Handled
    }

    static BAD_PARENT: rs_hsm_::SM_HsmState<BadSpec> = rs_hsm_::SM_HsmState {
        super_: None,
        init_: None,
        entry_: None,
        exit_: None,
        handler_: bad_state_handler,
    };

    static BAD_CHILD: rs_hsm_::SM_HsmState<BadSpec> = rs_hsm_::SM_HsmState {
        super_: Some(&BAD_PARENT),
        init_: None,
        entry_: None,
        exit_: Some(bad_child_exit),
        handler_: bad_state_handler,
    };

    static BAD_SIBLING: rs_hsm_::SM_HsmState<BadSpec> = rs_hsm_::SM_HsmState {
        super_: None,
        init_: None,
        entry_: None,
        exit_: None,
        handler_: bad_state_handler,
    };

    let mut ctx = BadObject { exited: false };
    let mut hsm = rs_hsm_::SM_Hsm::<BadSpec>::new();
    hsm.init(&mut ctx);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        hsm.transition(&mut ctx, &BAD_SIBLING, &BAD_PARENT);
    }));

    let payload = result.unwrap_err();
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&'static str>().copied())
        .expect("assert hook should panic with a message");
    assert!(message.contains("DBC assertion failed: rs_hsm_:310"));
    assert!(!ctx.exited);
}
