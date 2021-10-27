use std::path::{Path, PathBuf};

use clap::{Arg, SubCommand};

use file_reader::FileReader;
use run_command::RunCommand;

use crate::config::Config;
use crate::exit_codes::ExitCode;
use crate::results::basic_printer::BasicPrinter;
use crate::results::Printer;
use crate::runner::shell_executor::ShellExecutor;
use crate::runner::{Error, RunEvent};

mod exit_code;
mod file_reader;
mod run_command;

pub const NAME: &str = "run";

pub fn create() -> clap::App<'static, 'static> {
    let spec_file = Arg::with_name("spec-files")
        .index(1)
        .min_values(1)
        .help("The spec files to run")
        .required(true);

    let workspace_dir = Arg::with_name("workspace-dir")
        .long("workspace-dir")
        .takes_value(true)
        .value_name("dir")
        .help("Set the workspace directory")
        .required(false);

    let temp_workspace_dir = Arg::with_name("temp-workspace-dir")
        .long("temporary-workspace-dir")
        .takes_value(false)
        .help("Create a temporary workspace directory")
        .required(false);

    let working_dir = Arg::with_name("working-dir")
        .long("working-dir")
        .takes_value(true)
        .value_name("dir")
        .help(
            "The directory where commands will be executed. This is relative to the workspace dir",
        )
        .required(false);

    let workspace_init_command = Arg::with_name("workspace-init-command")
        .long("workspace-init-command")
        .takes_value(true)
        .value_name("command")
        .help("A command to run in the workspace before running the specs")
        .required(false);

    let shell_cmd = Arg::with_name("shell-command")
        .long("shell-command")
        .takes_value(true)
        .value_name("command")
        .default_value("bash -c")
        .help("The shell command used to execute script blocks")
        .required(false);

    let env = Arg::with_name("env")
        .long("env")
        .takes_value(true)
        .value_name("env-var")
        .multiple(true)
        .number_of_values(1)
        .help("Set an environment variable (format: 'VAR_NAME=value')")
        .required(false);

    let unset_env = Arg::with_name("unset-env")
        .long("unset-env")
        .takes_value(true)
        .value_name("var-name")
        .multiple(true)
        .number_of_values(1)
        .help("Unset an environment variable")
        .required(false);

    let add_path = Arg::with_name("add-path")
        .long("add-path")
        .takes_value(true)
        .value_name("path")
        .multiple(true)
        .number_of_values(1)
        .help("Adds the given directory to PATH")
        .required(false);

    SubCommand::with_name(NAME)
        .about("Runs a given Markdown Specification")
        .arg(spec_file)
        .arg(workspace_dir)
        .arg(temp_workspace_dir)
        .arg(working_dir)
        .arg(workspace_init_command)
        .arg(shell_cmd)
        .arg(env)
        .arg(unset_env)
        .arg(add_path)
}

pub fn execute(config: &Config, run_matches: &clap::ArgMatches<'_>) {
    let events = create_run_command(run_matches).map_or_else(
        |err| vec![RunEvent::ErrorOccurred(err)],
        |command| command.execute(),
    );

    let mut printer = BasicPrinter::new(config.colour);
    for event in &events {
        printer.print(event);
    }

    let exit_code = exit_code::from_events(&events);

    std::process::exit(exit_code as i32)
}

fn create_run_command(run_matches: &clap::ArgMatches<'_>) -> Result<RunCommand, Error> {
    let spec_files = run_matches
        .values_of("spec-files")
        .expect("spec-files should always exist")
        .map(Path::new)
        .map(std::path::Path::to_path_buf)
        .collect();
    let specified_workspace_dir = run_matches
        .value_of("workspace-dir")
        .map(Path::new)
        .map(std::path::Path::to_path_buf);
    let temp_workspace_dir = run_matches.is_present("temp-workspace-dir");
    let working_dir = run_matches
        .value_of("working-dir")
        .map(Path::new)
        .map(std::path::Path::to_path_buf);
    let workspace_init_command = run_matches
        .value_of("workspace-init-command")
        .map(std::string::ToString::to_string);
    let shell_cmd = run_matches.value_of("shell-command").unwrap().to_string();
    let mut env = run_matches
        .values_of("env")
        .map_or(vec![], parse_environment_variables);
    let unset_env = run_matches.values_of("unset-env").map_or(vec![], |v| {
        v.map(std::string::ToString::to_string).collect()
    });
    let paths = run_matches
        .values_of("add-path")
        .map_or(vec![], std::iter::Iterator::collect);
    let current_dir = std::env::current_dir().expect("Failed to get current workspace directory");
    let file_reader = FileReader::new(current_dir.clone());

    let workspace_dir = get_workspace_dir(specified_workspace_dir, temp_workspace_dir)
        .unwrap_or_else(|| current_dir.clone());

    std::fs::create_dir_all(&workspace_dir).expect("Failed to create workspace directory");
    let workspace_dir_canonicalized = std::fs::canonicalize(&workspace_dir)
        .unwrap_or_else(|_| panic!("Failed to canonicalize {:?}", workspace_dir));

    let actual_working_dir = working_dir.map_or_else(
        || workspace_dir_canonicalized.clone(),
        |dir| workspace_dir_canonicalized.clone().join(dir),
    );

    env.push((
        "SPECDOWN_START_DIR".to_string(),
        current_dir
            .into_os_string()
            .into_string()
            .expect("failed to convert start dir dir into a string"),
    ));

    env.push((
        "SPECDOWN_WORKSPACE_DIR".to_string(),
        workspace_dir_canonicalized
            .into_os_string()
            .into_string()
            .expect("failed to convert working dir into a string"),
    ));

    env.push((
        "SPECDOWN_WORKING_DIR".to_string(),
        actual_working_dir
            .clone()
            .into_os_string()
            .into_string()
            .expect("failed to convert working dir into a string"),
    ));

    let new_command = |e| RunCommand {
        spec_files,
        executor: Box::new(e),
        working_dir: actual_working_dir,
        workspace_init_command,
        file_reader,
    };

    ShellExecutor::new(&shell_cmd, &env, &unset_env, &paths).map(new_command)
}

fn get_workspace_dir(
    specified_workspace_dir: Option<PathBuf>,
    temp_workspace_dir: bool,
) -> Option<PathBuf> {
    if specified_workspace_dir.is_some() && temp_workspace_dir {
        println!(
            "  \u{2717} --workspace-dir and --temporary-workspace-dir cannot be specified at the same time"
        );
        std::process::exit(ExitCode::ErrorOccurred as i32)
    }

    if temp_workspace_dir {
        Some(
            tempfile::tempdir()
                .expect("Failed to create temporary workspace directory")
                .path()
                .to_path_buf(),
        )
    } else {
        specified_workspace_dir
    }
}

fn parse_environment_variables<'a>(
    strings: impl Iterator<Item = &'a str>,
) -> Vec<(String, String)> {
    strings.map(parse_environment_variable).collect()
}

fn parse_environment_variable(string: &str) -> (String, String) {
    match string.splitn(2, '=').collect::<Vec<_>>()[..] {
        [] => panic!("Empty environment variable split"),
        [name] => (name.to_string(), "".to_string()),
        [name, value, ..] => (name.to_string(), value.to_string()),
    }
}
