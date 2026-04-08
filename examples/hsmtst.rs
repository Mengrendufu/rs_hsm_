mod hsmtst_fixture;

use hsmtst_fixture::{SmHsmTst_run_sequence, SmHsmTst_select_sequence};
use std::env;

fn main() {
    let arg = env::args().nth(1);
    let run = SmHsmTst_run_sequence(SmHsmTst_select_sequence(arg.as_deref()));
    print!("{}", run.trace);
}
