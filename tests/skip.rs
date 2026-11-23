#![allow(dead_code)]

use getargs_derive::{Command, Error, GetArgs, Opt, Subcommand};

#[derive(Debug, PartialEq)]
struct Cache {
    hits: u32,
}

impl Default for Cache {
    fn default() -> Self {
        Cache { hits: 7 }
    }
}

// Neither `FromStr` nor `Copy`: a skipped field is never parsed.
#[derive(Debug, PartialEq)]
struct Session(Vec<u8>);

#[derive(Debug, GetArgs)]
#[command(name = "skip")]
struct Cli<'a> {
    /// Input file.
    input: &'a str,
    /// Internal cache, not an argument.
    #[arg(skip)]
    cache: Cache,
    #[arg(skip = 3)]
    retries: u8,
    #[arg(skip = "default")]
    profile: String,
    #[arg(skip = Session(vec![1, 2]))]
    session: Session,
    #[arg(skip)]
    count: Option<u32>,
    /// Print more output.
    #[arg(short, long)]
    verbose: bool,
}

#[test]
fn skipped_fields_get_their_default_or_expression() {
    let cli = Cli::from_args(["in.txt", "-v"].into_iter()).unwrap();
    assert_eq!(cli.input, "in.txt");
    assert!(cli.verbose);
    assert_eq!(cli.cache, Cache { hits: 7 });
    assert_eq!(cli.retries, 3);
    assert_eq!(cli.profile, "default");
    assert_eq!(cli.session, Session(vec![1, 2]));
    assert_eq!(cli.count, None);
}

#[test]
fn skipped_fields_are_not_arguments() {
    let error = Cli::from_args(["in.txt", "--cache", "1"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::UnknownOption {
            option: Opt::Long("cache"),
            suggestion: None,
        }
    );
    let error = Cli::from_args(["in.txt", "extra"].into_iter()).unwrap_err();
    assert_eq!(error, Error::UnexpectedArgument("extra"));
}

#[test]
fn skipped_fields_are_not_suggested() {
    let error = Cli::from_args(["in.txt", "--retrie"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::UnknownOption {
            option: Opt::Long("retrie"),
            suggestion: None,
        }
    );
}

#[test]
fn skipped_fields_are_not_in_the_help() {
    assert_eq!(
        Cli::HELP,
        "Usage: skip [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file

Options:
  -v, --verbose  Print more output
  -h, --help     Print help"
    );
}

#[test]
fn skipped_fields_are_not_in_the_command_metadata() {
    let ids: Vec<&str> = Cli::COMMAND.args.iter().map(|arg| arg.id).collect();
    assert_eq!(ids, ["input", "verbose", "help"]);
}

#[cfg(feature = "std")]
#[test]
fn skipped_fields_are_not_completed() {
    use getargs_derive::completions::{Shell, generate, render_man_page};
    for &shell in Shell::ALL {
        let mut output = Vec::new();
        generate::<Cli>(shell, &mut output).unwrap();
        let script = String::from_utf8(output).unwrap();
        assert!(script.contains("verbose"), "{shell} completes --verbose");
        for skipped in ["cache", "retries", "profile", "session", "count"] {
            assert!(!script.contains(skipped), "{shell} completes {skipped}");
        }
    }
    let page = render_man_page(&Cli::COMMAND);
    assert!(!page.contains("cache"));
}

#[derive(Debug, PartialEq, GetArgs)]
struct Verbosity {
    #[arg(short, long, action = Count)]
    verbose: u8,
    #[arg(skip = 2)]
    level: u8,
}

#[derive(Debug, GetArgs)]
struct Flattened {
    #[command(flatten)]
    verbosity: Verbosity,
    #[arg(skip)]
    name: String,
}

#[test]
fn skipped_fields_of_flattened_structs() {
    let cli = Flattened::from_args(["-vv"].into_iter()).unwrap();
    assert_eq!(
        cli.verbosity,
        Verbosity {
            verbose: 2,
            level: 2,
        }
    );
    assert_eq!(cli.name, "");
    let ids: Vec<&str> = Flattened::COMMAND.args.iter().map(|arg| arg.id).collect();
    assert_eq!(ids, ["verbose", "help"]);
}

#[derive(Debug, PartialEq, Subcommand)]
enum Commands<'a> {
    /// Copy a file.
    Copy {
        /// Source file.
        source: &'a str,
        #[arg(skip = 10u32)]
        attempts: u32,
        #[arg(skip)]
        cache: Cache,
    },
    Clean {
        #[arg(skip)]
        dry_run: bool,
    },
}

#[derive(Debug, GetArgs)]
#[command(name = "tool")]
struct Tool<'a> {
    #[command(subcommand)]
    command: Commands<'a>,
}

#[test]
fn skipped_fields_of_subcommand_variants() {
    let tool = Tool::from_args(["copy", "a.txt"].into_iter()).unwrap();
    assert_eq!(
        tool.command,
        Commands::Copy {
            source: "a.txt",
            attempts: 10,
            cache: Cache { hits: 7 },
        }
    );
    let tool = Tool::from_args(["clean"].into_iter()).unwrap();
    assert_eq!(tool.command, Commands::Clean { dry_run: false });
    let error = Tool::from_args(["clean", "--dry-run"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::UnknownOption {
            option: Opt::Long("dry-run"),
            suggestion: None,
        }
    );
}

#[test]
fn skipped_fields_are_not_in_the_subcommand_help() {
    let error = Tool::from_args(["copy", "-h"].into_iter()).unwrap_err();
    assert_eq!(
        error,
        Error::Help(
            "Copy a file

Usage: tool copy [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>  Source file

Options:
  -h, --help  Print help"
        )
    );
}
