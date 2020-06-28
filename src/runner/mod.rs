mod state;

use crate::results::printer::Printer;
use crate::results::test_result::TestResult;
use crate::runner::state::State;
use crate::types::Action;

mod error;
mod executor;
mod file;
mod script;
mod verify;

use executor::{Bash, Executor};

pub use error::Error;

pub fn run_actions(actions: &[Action], printer: &dyn Printer) {
    let mut state = State::new();
    let executor = Bash::new();

    let mut error_occurred = false;

    for action in actions {
        match run_action(action, &state, &executor) {
            Ok(result) => {
                state.add_result(&result);
                printer.print_result(&result)
            }
            Err(err) => {
                printer.print_error(&err);
                error_occurred = true;
                break;
            }
        }
    }

    if error_occurred {
        std::process::exit(2);
    }

    if !state.is_success() {
        std::process::exit(1);
    }
}

fn run_action(
    action: &Action,
    state: &State,
    executor: &dyn Executor,
) -> Result<TestResult, error::Error> {
    match action {
        Action::Script {
            script_name,
            script_code,
            expected_exit_code,
        } => script::run(script_name, script_code, expected_exit_code, executor),
        Action::Verify {
            source,
            expected_value,
        } => verify::run(source, expected_value, state),
        Action::CreateFile {
            file_path,
            file_content,
        } => file::run(file_path, file_content),
    }
}
