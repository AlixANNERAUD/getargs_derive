#![allow(dead_code)]

use core::num::ParseIntError;
use getargs_derive::{Error, GetArgs, Subcommand, Values};

#[derive(Debug, PartialEq)]
struct Port(u16);

fn parse_port(value: &str) -> Result<Port, ParseIntError> {
    value.parse().map(Port)
}

fn non_empty(value: &str) -> Result<&str, ()> {
    match value.trim() {
        "" => Err(()),
        trimmed => Ok(trimmed),
    }
}

mod parsers {
    pub fn even(value: &str) -> Result<u32, ()> {
        match value.parse::<u32>() {
            Ok(number) if number % 2 == 0 => Ok(number),
            _ => Err(()),
        }
    }
}

#[derive(Debug, GetArgs)]
#[command(name = "validation")]
struct Validated<'a> {
    #[arg(value_parser = non_empty)]
    name: &'a str,
    #[arg(short, long, value_parser = parse_port, default_value = "8080")]
    port: Port,
    #[arg(short, long, value_parser = 1..=64, default_value_t = 4)]
    jobs: u8,
    #[arg(long, value_parser = parsers::even)]
    even: Option<u32>,
    #[arg(short, long, possible_values = ["fast", "slow"], default_value = "fast")]
    mode: &'a str,
    #[arg(long, possible_values = ["1", "2", "4"])]
    level: Option<u8>,
}

#[test]
fn defaults_go_through_validation() {
    let params = Validated::from_args(["app"].into_iter()).unwrap();
    assert_eq!(params.name, "app");
    assert_eq!(params.port, Port(8080));
    assert_eq!(params.jobs, 4);
    assert_eq!(params.even, None);
    assert_eq!(params.mode, "fast");
    assert_eq!(params.level, None);
}

#[test]
fn value_parser_functions() {
    let args = [" app ", "-p", "3000", "--even", "6"];
    let params = Validated::from_args(args.into_iter()).unwrap();
    assert_eq!(params.name, "app");
    assert_eq!(params.port, Port(3000));
    assert_eq!(params.even, Some(6));

    let err = Validated::from_args(["  "].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "<NAME>",
            value: "  "
        }
    );

    let err = Validated::from_args(["app", "--port", "http"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--port <PORT>",
            value: "http"
        }
    );

    let err = Validated::from_args(["app", "--even", "3"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--even <EVEN>",
            value: "3"
        }
    );
}

#[test]
fn ranges() {
    let params = Validated::from_args(["app", "-j", "1"].into_iter()).unwrap();
    assert_eq!(params.jobs, 1);
    let params = Validated::from_args(["app", "-j", "64"].into_iter()).unwrap();
    assert_eq!(params.jobs, 64);

    let err = Validated::from_args(["app", "-j", "0"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ValueOutOfRange {
            name: "--jobs <JOBS>",
            value: "0",
            range: "1..=64"
        }
    );
    assert_eq!(
        err.to_string(),
        "invalid value '0' for '--jobs <JOBS>': 0 is not in 1..=64"
    );

    let err = Validated::from_args(["app", "-j", "65"].into_iter()).unwrap_err();
    assert!(matches!(err, Error::ValueOutOfRange { value: "65", .. }));

    let err = Validated::from_args(["app", "-j", "x"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--jobs <JOBS>",
            value: "x"
        }
    );
}

#[test]
fn possible_values() {
    let params = Validated::from_args(["app", "--mode=slow", "--level", "4"].into_iter()).unwrap();
    assert_eq!(params.mode, "slow");
    assert_eq!(params.level, Some(4));

    let err = Validated::from_args(["app", "-m", "turbo"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidPossibleValue {
            name: "--mode <MODE>",
            value: "turbo",
            possible_values: &["fast", "slow"],
        }
    );
    assert_eq!(
        err.to_string(),
        "invalid value 'turbo' for '--mode <MODE>'\n  [possible values: fast, slow]"
    );

    let err = Validated::from_args(["app", "--level", "04"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidPossibleValue {
            name: "--level <LEVEL>",
            value: "04",
            possible_values: &["1", "2", "4"],
        }
    );
}

#[derive(Debug, GetArgs)]
struct Ranges {
    #[arg(long, value_parser = 1..)]
    from: Option<u32>,
    #[arg(long, value_parser = ..10)]
    to: Option<u32>,
    #[arg(long, value_parser = ..=-1)]
    negative: Option<i8>,
    #[arg(long, value_parser = -5..5)]
    window: Option<i64>,
    #[arg(long, value_parser = 1..=8, default_value = "9")]
    invalid_default: u8,
}

#[test]
fn range_forms() {
    let args = [
        "--from",
        "7",
        "--to",
        "9",
        "--negative=-3",
        "--window=-5",
        "--invalid-default",
        "8",
    ];
    let params = Ranges::from_args(args.into_iter()).unwrap();
    assert_eq!(params.from, Some(7));
    assert_eq!(params.to, Some(9));
    assert_eq!(params.negative, Some(-3));
    assert_eq!(params.window, Some(-5));
    assert_eq!(params.invalid_default, 8);

    let cases: [(&[&str], &str); 4] = [
        (&["--from", "0"], "1.."),
        (&["--to", "10"], "..10"),
        (&["--negative", "0"], "..=-1"),
        (&["--window", "5"], "-5..5"),
    ];
    for (args, expected) in cases {
        let args = args.iter().copied().chain(["--invalid-default", "1"]);
        match Ranges::from_args(args).unwrap_err() {
            Error::ValueOutOfRange { range, .. } => assert_eq!(range, expected),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    let err = Ranges::from_args([].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::ValueOutOfRange {
            name: "--invalid-default <INVALID_DEFAULT>",
            value: "9",
            range: "1..=8"
        }
    );
}

#[derive(Debug, GetArgs)]
#[command(name = "validation")]
struct Combined<'a> {
    #[arg(possible_values = ["2", "3", "20"], value_parser = 1..=10)]
    count: u32,
    #[arg(long, possible_values = ["a b", "c"], value_parser = non_empty)]
    spaced: Option<&'a str>,
}

#[test]
fn combined_validation_order() {
    let params = Combined::from_args(["3", "--spaced", "a b"].into_iter()).unwrap();
    assert_eq!(params.count, 3);
    assert_eq!(params.spaced, Some("a b"));

    let err = Combined::from_args(["5"].into_iter()).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidPossibleValue { value: "5", .. }
    ));

    let err = Combined::from_args(["20"].into_iter()).unwrap_err();
    assert!(matches!(err, Error::ValueOutOfRange { value: "20", .. }));

    let err = Combined::from_args(["2", "--spaced", "d"].into_iter()).unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid value 'd' for '--spaced <SPACED>'\n  [possible values: \"a b\", c]"
    );
}

#[test]
fn help_lists_possible_values() {
    assert_eq!(
        Validated::HELP,
        "Usage: validation [OPTIONS] <NAME>

Arguments:
  <NAME>

Options:
  -p, --port <PORT>    [default: 8080]
  -j, --jobs <JOBS>    [default: 4]
      --even <EVEN>
  -m, --mode <MODE>    [default: fast] [possible values: fast, slow]
      --level <LEVEL>  [possible values: 1, 2, 4]
  -h, --help           Print help"
    );
    assert_eq!(
        Combined::HELP,
        "Usage: validation [OPTIONS] <COUNT>

Arguments:
  <COUNT>  [possible values: 2, 3, 20]

Options:
      --spaced <SPACED>  [possible values: \"a b\", c]
  -h, --help             Print help"
    );
}

#[derive(Debug, GetArgs)]
struct FromEnv<'a> {
    #[arg(long, env = "JOBS", value_parser = 1..=64)]
    jobs: Option<u8>,
    #[arg(long, env = "MODE", possible_values = ["fast", "slow"])]
    mode: Option<&'a str>,
}

#[test]
fn env_values_are_validated() {
    let vars = [("JOBS", "8"), ("MODE", "slow")];
    let lookup = |name: &str| {
        vars.iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
    };
    let cli = FromEnv::from_args_with_env([].into_iter(), lookup).unwrap();
    assert_eq!(cli.jobs, Some(8));
    assert_eq!(cli.mode, Some("slow"));

    let err = FromEnv::from_args_with_env([].into_iter(), |name: &str| {
        (name == "JOBS").then_some("65")
    })
    .unwrap_err();
    assert_eq!(
        err,
        Error::ValueOutOfRange {
            name: "--jobs <JOBS> [env: JOBS]",
            value: "65",
            range: "1..=64"
        }
    );

    let err = FromEnv::from_args_with_env([].into_iter(), |name: &str| {
        (name == "MODE").then_some("turbo")
    })
    .unwrap_err();
    assert_eq!(
        err,
        Error::InvalidPossibleValue {
            name: "--mode <MODE> [env: MODE]",
            value: "turbo",
            possible_values: &["fast", "slow"]
        }
    );
}

fn unprivileged_port(value: &str) -> Result<u16, &'static str> {
    match value.parse::<u16>() {
        Ok(port) if port >= 1024 => Ok(port),
        Ok(_) => Err("must be at least 1024"),
        Err(_) => Err("not a port number"),
    }
}

// Elided lifetimes tie the error to the value: it is kept as the reason too.
fn no_dots(value: &str) -> Result<&str, &str> {
    match value.find('.') {
        Some(index) => Err(&value[index..]),
        None => Ok(value),
    }
}

type Reason = &'static str;

fn even(value: &str) -> Result<u32, Reason> {
    match value.parse::<u32>() {
        Ok(number) if number % 2 == 0 => Ok(number),
        _ => Err("must be an even number"),
    }
}

fn owned_reason(value: &str) -> Result<u8, &'static &'static str> {
    value.parse().map_err(|_| &"never shown")
}

#[derive(Debug, GetArgs)]
#[command(name = "reasons")]
struct Reasons<'a> {
    #[arg(short, long, value_parser = unprivileged_port, default_value = "8080")]
    port: u16,
    #[arg(long, value_parser = no_dots)]
    name: Option<&'a str>,
    #[arg(long, value_parser = even, env = "REASONS_EVEN")]
    even: Option<u32>,
    #[arg(long, value_parser = owned_reason)]
    other: Option<u8>,
    #[arg(long, value_parser = parse_port)]
    legacy: Option<Port>,
    #[arg(short = 'I', long, value_parser = unprivileged_port, value_delimiter = ',')]
    include: Values<u16, 4>,
}

#[test]
fn str_errors_of_value_parsers_become_the_reason() {
    let params = Reasons::from_args_with_env(["-p", "3000"].into_iter(), |_| None).unwrap();
    assert_eq!(params.port, 3000);

    let err = Reasons::from_args_with_env(["-p", "80"].into_iter(), |_| None).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValueReason {
            name: "--port <PORT>",
            value: "80",
            reason: "must be at least 1024",
        }
    );
    assert_eq!(
        err.to_string(),
        "invalid value '80' for '--port <PORT>': must be at least 1024"
    );

    let err = Reasons::from_args_with_env(["--name", "a.b"].into_iter(), |_| None).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValueReason {
            name: "--name <NAME>",
            value: "a.b",
            reason: ".b",
        }
    );

    let err = Reasons::from_args_with_env(["--even", "3"].into_iter(), |_| None).unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid value '3' for '--even <EVEN>': must be an even number"
    );
}

#[test]
fn other_errors_of_value_parsers_are_discarded() {
    let err = Reasons::from_args_with_env(["--other", "x"].into_iter(), |_| None).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--other <OTHER>",
            value: "x",
        }
    );
    let err = Reasons::from_args_with_env(["--legacy", "x"].into_iter(), |_| None).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValue {
            name: "--legacy <LEGACY>",
            value: "x",
        }
    );
}

#[test]
fn reasons_for_env_defaults_and_collections() {
    let err = Reasons::from_args_with_env([].into_iter(), |name: &str| {
        (name == "REASONS_EVEN").then_some("5")
    })
    .unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValueReason {
            name: "--even <EVEN> [env: REASONS_EVEN]",
            value: "5",
            reason: "must be an even number",
        }
    );

    let err = Reasons::from_args_with_env(["-I", "2000,22"].into_iter(), |_| None).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValueReason {
            name: "--include <INCLUDE>",
            value: "22",
            reason: "must be at least 1024",
        }
    );
}

#[derive(Debug, GetArgs)]
struct BadDefault {
    #[arg(long, value_parser = unprivileged_port, default_value = "21")]
    port: u16,
}

#[test]
fn reasons_for_default_values() {
    let err = BadDefault::from_args([].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValueReason {
            name: "--port <PORT>",
            value: "21",
            reason: "must be at least 1024",
        }
    );
}

#[test]
fn reasons_with_usage() {
    let err = Reasons::from_args_with_env(["-p", "80"].into_iter(), |_| None).unwrap_err();
    assert_eq!(
        err.with_usage(Reasons::USAGE).to_string(),
        "invalid value '80' for '--port <PORT>': must be at least 1024

Usage: reasons [OPTIONS]

For more information, try '--help'."
    );
}

#[derive(Debug, PartialEq, Subcommand)]
enum Commands {
    Serve {
        #[arg(long, value_parser = unprivileged_port)]
        port: u16,
    },
}

#[derive(Debug, GetArgs)]
struct Server {
    #[command(subcommand)]
    command: Commands,
}

#[test]
fn reasons_in_subcommand_variants() {
    let server = Server::from_args(["serve", "--port", "4000"].into_iter()).unwrap();
    assert_eq!(server.command, Commands::Serve { port: 4000 });
    let err = Server::from_args(["serve", "--port", "x"].into_iter()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValueReason {
            name: "--port <PORT>",
            value: "x",
            reason: "not a port number",
        }
    );
}

#[test]
fn reasons_keep_the_error_small() {
    // Three two-word fields, like `InvalidPossibleValue`: the new variant must not grow `Error`.
    #[cfg(target_pointer_width = "64")]
    assert_eq!(core::mem::size_of::<Error<'static>>(), 56);
}
