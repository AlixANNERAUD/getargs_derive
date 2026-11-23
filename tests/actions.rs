#![allow(dead_code)]

use getargs_derive::{ArgKind, Command, Error, GetArgs, Opt, Subcommand};

#[derive(Debug, GetArgs)]
struct FlagActions {
    #[arg(short, long, action = Count)]
    verbose: u8,
    #[arg(short = 'q', action = ArgAction::Count)]
    quiet: u32,
    #[arg(long, action = SetFalse)]
    no_color: bool,
    #[arg(long = "fast", action = SetTrue)]
    fast: bool,
    #[arg(long, action = Count, default_value_t = 3)]
    level: i64,
}

#[test]
fn flag_actions_defaults() {
    let params = FlagActions::from_args([].into_iter()).unwrap();
    assert_eq!(params.verbose, 0);
    assert_eq!(params.quiet, 0);
    assert!(params.no_color);
    assert!(!params.fast);
    assert_eq!(params.level, 3);
}

#[test]
fn count_action_counts_occurrences() {
    let args = ["-vvv", "--verbose", "-q", "-vq", "--level", "--level"];
    let params = FlagActions::from_args(args.into_iter()).unwrap();
    assert_eq!(params.verbose, 5);
    assert_eq!(params.quiet, 2);
    assert_eq!(params.level, 2);
}

#[test]
fn count_action_saturates() {
    let args = ["-v"; 300];
    let params = FlagActions::from_args(args.into_iter()).unwrap();
    assert_eq!(params.verbose, u8::MAX);
}

#[test]
fn set_false_and_set_true_actions() {
    let params = FlagActions::from_args(["--no-color", "--fast"].into_iter()).unwrap();
    assert!(!params.no_color);
    assert!(params.fast);
}

#[test]
fn count_action_rejects_value() {
    let err = FlagActions::from_args(["--verbose=2"].into_iter()).unwrap_err();
    assert_eq!(err, Error::UnexpectedValue(Opt::Long("verbose")));
}

#[derive(Debug, GetArgs)]
#[command(name = "actions")]
struct HelpActions {
    /// More output
    #[arg(short, long, action = Count)]
    verbose: u8,
    #[arg(short = 'q', action = Count, default_value_t = 1)]
    quiet: u8,
    /// Disable colors
    #[arg(long, action = SetFalse)]
    color: bool,
}

#[test]
fn help_actions_snapshot() {
    assert_eq!(
        HelpActions::HELP,
        "Usage: actions [OPTIONS]

Options:
  -v, --verbose...  More output
  -q...
      --color       Disable colors
  -h, --help        Print help"
    );
}

fn yes_no(value: &str) -> Result<bool, ()> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(()),
    }
}

fn lookup<'a>(vars: &'a [(&'a str, &'a str)]) -> impl FnMut(&str) -> Option<&'a str> {
    move |name| {
        vars.iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
    }
}

#[derive(Debug, GetArgs)]
#[command(name = "set")]
struct SetActions {
    /// Advertise the service with DNS-SD.
    #[arg(long, action = Set)]
    dnssd: bool,
    /// Use colors.
    #[arg(short, long, action = ArgAction::Set, default_value_t = true)]
    color: bool,
    #[arg(long, action = Set, env = "SET_CACHE", default_value = "false", value_name = "BOOL")]
    cache: bool,
    /// Ask before overwriting.
    #[arg(long, action = Set, value_parser = yes_no, default_value = "no")]
    confirm: bool,
    /// Print more output.
    #[arg(short, long)]
    verbose: bool,
}

#[test]
fn set_action_takes_a_value() {
    let args = ["--dnssd", "true", "-c", "false", "--cache=true", "-v"];
    let params = SetActions::from_args_with_env(args.into_iter(), lookup(&[])).unwrap();
    assert!(params.dnssd);
    assert!(!params.color);
    assert!(params.cache);
    assert!(!params.confirm);
    assert!(params.verbose);

    let params =
        SetActions::from_args_with_env(["--dnssd=false"].into_iter(), lookup(&[])).unwrap();
    assert!(!params.dnssd);
    assert!(params.color);
    assert!(!params.cache);
}

#[test]
fn set_action_is_required_without_default() {
    let error = SetActions::from_args_with_env([].into_iter(), lookup(&[])).unwrap_err();
    assert_eq!(error, Error::MissingArgument("--dnssd <DNSSD>"));
    let error = SetActions::from_args_with_env(["--dnssd"].into_iter(), lookup(&[])).unwrap_err();
    assert_eq!(error, Error::MissingOptionValue("--dnssd <DNSSD>"));
}

#[test]
fn set_action_accepts_only_true_and_false() {
    for value in ["yes", "True", "1", ""] {
        let error = SetActions::from_args_with_env(["--dnssd", value].into_iter(), lookup(&[]))
            .unwrap_err();
        assert_eq!(
            error,
            Error::InvalidPossibleValue {
                name: "--dnssd <DNSSD>",
                value,
                possible_values: &["true", "false"],
            }
        );
    }
    let error =
        SetActions::from_args_with_env(["--dnssd", "on"].into_iter(), lookup(&[])).unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid value 'on' for '--dnssd <DNSSD>'\n  [possible values: true, false]"
    );
}

#[test]
fn set_action_reads_the_environment() {
    let args = ["--dnssd", "true"];
    let params =
        SetActions::from_args_with_env(args.into_iter(), lookup(&[("SET_CACHE", "true")])).unwrap();
    assert!(params.cache);
    let params = SetActions::from_args_with_env(
        ["--dnssd", "true", "--cache", "false"].into_iter(),
        lookup(&[("SET_CACHE", "true")]),
    )
    .unwrap();
    assert!(!params.cache);
    let error = SetActions::from_args_with_env(args.into_iter(), lookup(&[("SET_CACHE", "on")]))
        .unwrap_err();
    assert_eq!(
        error,
        Error::InvalidPossibleValue {
            name: "--cache <BOOL> [env: SET_CACHE]",
            value: "on",
            possible_values: &["true", "false"],
        }
    );
}

#[test]
fn set_action_with_value_parser() {
    let args = ["--dnssd", "true", "--confirm", "yes"];
    let params = SetActions::from_args_with_env(args.into_iter(), lookup(&[])).unwrap();
    assert!(params.confirm);
    let args = ["--dnssd", "true", "--confirm", "true"];
    let error = SetActions::from_args_with_env(args.into_iter(), lookup(&[])).unwrap_err();
    assert_eq!(
        error,
        Error::InvalidValue {
            name: "--confirm <CONFIRM>",
            value: "true",
        }
    );
}

#[test]
fn set_action_help_snapshot() {
    assert_eq!(
        SetActions::HELP,
        "Usage: set [OPTIONS]

Options:
      --dnssd <DNSSD>      Advertise the service with DNS-SD [possible values: true, false]
  -c, --color <COLOR>      Use colors [default: true] [possible values: true, false]
      --cache <BOOL>       [env: SET_CACHE=] [default: false] [possible values: true, false]
      --confirm <CONFIRM>  Ask before overwriting [default: no]
  -v, --verbose            Print more output
  -h, --help               Print help"
    );
}

#[test]
fn set_action_metadata() {
    let args = SetActions::COMMAND.args;
    assert_eq!(args[0].kind, ArgKind::Option);
    assert_eq!(args[0].value_name, "DNSSD");
    assert_eq!(args[0].possible_values, ["true", "false"]);
    assert!(args[0].required);
    assert_eq!(args[2].env, Some("SET_CACHE"));
    assert!(!args[2].required);
    assert!(args[3].possible_values.is_empty());
    assert_eq!(args[4].kind, ArgKind::Flag);
}

#[derive(Debug, PartialEq, Subcommand)]
enum Commands {
    Serve {
        #[arg(long, action = Set, default_value_t = false)]
        dnssd: bool,
    },
}

#[derive(Debug, GetArgs)]
struct Tool {
    #[command(subcommand)]
    command: Commands,
}

#[test]
fn set_action_in_subcommand_variants() {
    let tool = Tool::from_args(["serve", "--dnssd", "true"].into_iter()).unwrap();
    assert_eq!(tool.command, Commands::Serve { dnssd: true });
    let tool = Tool::from_args(["serve"].into_iter()).unwrap();
    assert_eq!(tool.command, Commands::Serve { dnssd: false });
    let error = Tool::from_args(["serve", "--dnssd", "no"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::InvalidPossibleValue {
            name: "--dnssd <DNSSD>",
            value: "no",
            possible_values: &["true", "false"],
        }
    );
}
