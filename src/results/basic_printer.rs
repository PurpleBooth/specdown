use super::printer::Printer;
use super::test_result::TestResult;
use crate::runner::Error;

pub struct BasicPrinter {
    display: Box<dyn Fn(&str)>,
}

impl BasicPrinter {
    pub fn new() -> Self {
        Self {
            display: Box::new(|line| println!("{}", line)),
        }
    }
}

impl Printer for BasicPrinter {
    fn print_result(&self, result: &TestResult) {
        let display = &self.display;
        match result {
            TestResult::Script {
                name,
                success,
                expected_exit_code,
                exit_code,
                stdout,
                stderr,
                ..
            } => {
                let message = if *success {
                    "succeeded".to_string()
                } else {
                    let expected =
                        expected_exit_code.map_or("None".to_string(), |code| code.to_string());
                    let got = exit_code.map_or("None".to_string(), |code| code.to_string());

                    format!(
                        "failed (expected exitcode {}, got {})\n+++ stdout:\n{}\n\n+++ stderr:\n{}\n\n",
                        expected, got, stdout, stderr
                    )
                };
                display(&format!("- script '{}' {}", name, message))
            }
            TestResult::Verify {
                script_name,
                success,
                expected,
                got,
                ..
            } => {
                display(&format!(
                    "- verify output from '{}' {}",
                    script_name,
                    if *success { "succeeded" } else { "failed" }
                ));

                if !*success {
                    display(&format!(
                        "{}",
                        colored_diff::PrettyDifference {
                            expected,
                            actual: got
                        }
                    ));
                }
            }
            TestResult::File { path } => display(&format!("File {} created", path)),
        }
    }

    fn print_error(&self, error: &Error) {
        let display = &self.display;
        match error {
            Error::ScriptOutputMissing {
                missing_script_name,
            } => {
                display(&format!(
                    "Failed to verify the output of '{}': No script with that name has been executed yet.",
                    missing_script_name
                ));
            }
            Error::CommandFailed => panic!("Failed to run script"),
        }
    }
}

#[cfg(test)]
mod tests {}
