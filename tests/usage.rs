#![allow(dead_code)]

use getargs_derive::{Error, GetArgs, Subcommand, Usage, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum Format {
    Espressif,
    Mbr,
    Linker,
    #[value(name = "constants", alias = "c")]
    Constants,
}

#[derive(Debug, GetArgs)]
#[command(name = "layout")]
struct Write<'a> {
    #[arg(short, long, value_enum)]
    format: Format,
    input: &'a str,
}

#[derive(Debug, GetArgs)]
#[command(name = "plain", disable_help_flag)]
struct NoHelp<'a> {
    input: &'a str,
}

/// Copy files around.
#[derive(Debug, GetArgs)]
#[command(
    name = "copy",
    before_help = "Before the help",
    long_about = "A longer description of what copy does.",
    after_help = "After the help"
)]
struct Described<'a> {
    #[arg(short, long)]
    verbose: bool,
    input: &'a str,
}

#[derive(Debug, GetArgs)]
struct Common {
    #[arg(short, long)]
    quiet: bool,
}

/// Flattens another struct.
#[derive(Debug, GetArgs)]
#[command(name = "flat")]
struct Flat<'a> {
    #[command(flatten)]
    common: Common,
    input: &'a str,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Check,
    Write,
}

/// A layout tool.
#[derive(Debug, GetArgs)]
#[command(name = "layout")]
struct Tool {
    #[command(subcommand)]
    command: Commands,
}

#[test]
fn value_enum_errors_list_the_possible_values() {
    let error = Write::from_args(["--format", "bogus", "in"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::InvalidPossibleValue {
            name: "--format <FORMAT>",
            value: "bogus",
            possible_values: &["espressif", "mbr", "linker", "constants"],
        }
    );
    assert_eq!(
        error.to_string(),
        "invalid value 'bogus' for '--format <FORMAT>'\n  [possible values: espressif, mbr, linker, constants]"
    );
}

#[test]
fn missing_subcommand_leaves_a_blank_line() {
    let error = Tool::from_args([].into_iter()).unwrap_err();
    assert_eq!(
        error.to_string(),
        "a subcommand is required but one was not provided\n\n  [subcommands: check, write]"
    );
}

#[test]
fn usage_is_the_usage_line_of_the_help() {
    assert_eq!(Write::USAGE.line, "Usage: layout [OPTIONS] <INPUT>");
    assert_eq!(Write::USAGE.to_string(), "Usage: layout [OPTIONS] <INPUT>");
    assert_eq!(Write::USAGE, Usage::new("Usage: layout [OPTIONS] <INPUT>"));
    assert_eq!(NoHelp::USAGE.line, "Usage: plain <INPUT>");
    assert_eq!(
        NoHelp::USAGE,
        Usage::new("Usage: plain <INPUT>").help_flag(false)
    );
    assert_eq!(Described::USAGE.line, "Usage: copy [OPTIONS] <INPUT>");
    assert_eq!(Flat::USAGE.line, "Usage: flat [OPTIONS] <INPUT>");
    assert_eq!(Tool::USAGE.line, "Usage: layout [OPTIONS] <COMMAND>");
    for (usage, help) in [
        (Write::USAGE, Write::HELP),
        (Described::USAGE, Described::HELP),
        (Flat::USAGE, Flat::HELP),
        (Tool::USAGE, Tool::HELP),
    ] {
        assert_eq!(usage.line, help_usage(help));
    }
}

fn help_usage(help: &str) -> &str {
    help.lines()
        .find(|line| line.starts_with("Usage: "))
        .unwrap()
}

#[test]
fn errors_end_with_the_usage_and_a_help_hint() {
    let error = Write::from_args(["--format", "bogus", "in"].into_iter()).unwrap_err();
    assert_eq!(
        error.with_usage(Write::USAGE).to_string(),
        "invalid value 'bogus' for '--format <FORMAT>'\n  [possible values: espressif, mbr, linker, constants]\n\nUsage: layout [OPTIONS] <INPUT>\n\nFor more information, try '--help'."
    );
    let error = Tool::from_args([].into_iter()).unwrap_err();
    assert_eq!(
        error.with_usage(Tool::USAGE).to_string(),
        "a subcommand is required but one was not provided\n\n  [subcommands: check, write]\n\nUsage: layout [OPTIONS] <COMMAND>\n\nFor more information, try '--help'."
    );
}

#[test]
fn the_hint_is_left_out_without_a_help_flag() {
    let error = NoHelp::from_args([].into_iter()).unwrap_err();
    assert_eq!(
        error.with_usage(NoHelp::USAGE).to_string(),
        "the following required argument was not provided: <INPUT>\n\nUsage: plain <INPUT>"
    );
}

#[test]
fn help_and_version_are_written_alone() {
    let error = Write::from_args(["-h"].into_iter()).unwrap_err();
    let report = error.with_usage(Write::USAGE);
    assert_eq!(report.to_string(), Write::HELP);
    assert_eq!(report.error(), &error);
    assert_eq!(report.usage(), &Write::USAGE);
}

#[test]
fn a_usage_can_be_built_by_hand() {
    let usage = Usage::new("Usage: tool").help_flag(false);
    assert_eq!(usage.line, "Usage: tool");
    let error = Error::UnexpectedArgument("x");
    assert_eq!(
        error.with_usage(usage).to_string(),
        "unexpected argument 'x' found\n\nUsage: tool"
    );
}
