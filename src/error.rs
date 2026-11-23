/// An option as it was written on the command line, without its leading dashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opt<'a> {
    /// A short option, like `-z` (stored as `'z'`).
    Short(char),
    /// A long option, like `--bogus` (stored as `"bogus"`).
    Long(&'a str),
}

impl core::fmt::Display for Opt<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Opt::Short(short) => write!(f, "-{short}"),
            Opt::Long(long) => write!(f, "--{long}"),
        }
    }
}

/// Errors that can occur when parsing CLI arguments with [`GetArgs`](crate::GetArgs).
///
/// The lifetime `'a` is the lifetime of the parsed arguments: variants describing what the
/// user typed borrow it from the command line instead of copying it, so no allocation is needed.
///
/// Names of the struct's own arguments (`&'static str`) are written as in the generated help:
/// `<INPUT>` for a positional argument, `--output <OUTPUT>` for an option taking a value
/// (`-o <OUTPUT>` when it has no long name).
///
/// The enum is `#[non_exhaustive]`: new variants can be added in a minor release, so a `match` on
/// it needs a wildcard arm.
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<'a> {
    /// A required positional argument or named option was not provided on the command line.
    ///
    /// The string is the argument name, e.g. `"<INPUT>"` or `"--output <OUTPUT>"`.
    MissingArgument(&'static str),

    /// An option that the struct does not define was given, e.g. `--bogus` or `-z`.
    UnknownOption {
        /// The unknown option, as written by the user.
        option: Opt<'a>,
        /// The closest long option name (without `--`) when an unknown long option looks like a
        /// typo of a known one, e.g. `Some("verbose")` for `--verbos`.
        suggestion: Option<&'static str>,
    },

    /// A positional argument was given but the struct expects no more of them.
    ///
    /// The string is the extra argument, as written by the user.
    UnexpectedArgument(&'a str),

    /// A value was attached to an option that does not take one, e.g. `--verbose=yes` for a flag.
    ///
    /// The option is the one written by the user.
    UnexpectedValue(Opt<'a>),

    /// An option taking a value was given without one, e.g. `--output` at the end of the
    /// command line.
    ///
    /// The string is the option name, e.g. `"--output <OUTPUT>"`.
    MissingOptionValue(&'static str),

    /// Two arguments that cannot be used together were both given, because of `conflicts_with`
    /// or because they belong to the same group.
    ArgumentConflict {
        /// The argument declaring the conflict (or the first member of the group), e.g.
        /// `"--quiet"`.
        name: &'static str,
        /// The other argument, e.g. `"--verbose"`.
        other: &'static str,
    },

    /// An argument was given without another argument it `requires`.
    MissingRequiredArgument {
        /// The missing argument, e.g. `"--output <OUTPUT>"`.
        name: &'static str,
        /// The given argument that requires it, e.g. `"--compress"`.
        required_by: &'static str,
    },

    /// None of a set of arguments was given although one of them is required, because of
    /// `required_unless_present` or a `required` group.
    ///
    /// The slice lists the arguments, e.g. `["--config <CONFIG>", "--stdin"]`.
    MissingOneOf(&'static [&'static str]),

    /// A required subcommand was not given.
    ///
    /// The slice lists the visible subcommands, e.g. `["add", "remove"]`. It is only returned for
    /// a `#[command(subcommand)]` field that is not an `Option`.
    MissingSubcommand(&'static [&'static str]),

    /// A positional argument was given where a subcommand was expected, and it is not the name
    /// or an alias of any subcommand.
    UnknownSubcommand {
        /// The unknown subcommand, as written by the user.
        name: &'a str,
        /// The closest visible subcommand name or alias when the unknown one looks like a typo of
        /// it, e.g. `Some("remove")` for `remov`.
        suggestion: Option<&'static str>,
    },

    /// The underlying `getargs` parser reported an error that this crate does not recognise.
    ///
    /// `getargs` may add error kinds in a compatible release; they are reported as this variant
    /// instead of panicking. It is never returned with the `getargs` 0.5 releases known to this
    /// crate.
    InvalidArguments,

    /// A value could not be parsed into the target field type, e.g. `abc` for a `u32` field.
    ///
    /// This is also returned when a `default_value` string does not parse.
    InvalidValue {
        /// The argument name, e.g. `"--port <PORT>"` or `"<COUNT>"`.
        name: &'static str,
        /// The value that failed to parse.
        value: &'a str,
    },

    /// A value is not one of the argument's `possible_values`, e.g. `turbo` for
    /// `possible_values = ["fast", "slow"]`.
    ///
    /// This is also returned when a `default_value` string is not one of them.
    InvalidPossibleValue {
        /// The argument name, e.g. `"--mode <MODE>"`.
        name: &'static str,
        /// The value that was given.
        value: &'a str,
        /// The accepted values, in declaration order, e.g. `["fast", "slow"]`.
        possible_values: &'static [&'static str],
    },

    /// A value parsed into the field type but is outside the argument's `value_parser` range,
    /// e.g. `0` for `value_parser = 1..=64`.
    ///
    /// This is also returned when a `default_value` string is outside the range.
    ValueOutOfRange {
        /// The argument name, e.g. `"--jobs <JOBS>"`.
        name: &'static str,
        /// The value that was given.
        value: &'a str,
        /// The range as written in the attribute, e.g. `"1..=64"`.
        range: &'static str,
    },

    /// More values were given to a repeated argument than its collection can hold, or than its
    /// `num_args` range accepts, e.g. a third `-I` for a `Values<&str, 2>` field.
    ///
    /// This is also returned when an environment variable or a default splits into too many
    /// values.
    TooManyValues {
        /// The argument name, e.g. `"--include <INCLUDE>"` or `"<FILES>..."`.
        name: &'static str,
        /// The first value that did not fit.
        value: &'a str,
        /// The maximum number of values.
        max: usize,
    },

    /// A positional argument collecting several values received fewer than the minimum of its
    /// `num_args` range, but at least one. [`Error::MissingArgument`] is returned when it received
    /// none.
    TooFewValues {
        /// The argument name, e.g. `"<FILES>..."`.
        name: &'static str,
        /// The minimum number of values.
        min: usize,
        /// The number of values given.
        actual: usize,
    },

    /// An argument that is not valid UTF-8 was given where text is required, when parsing raw
    /// bytes with `from_bytes`, `parse_bytes` or `from_args_os`.
    ///
    /// It is returned for a value of a `&str` field or of a field parsed from a string, and for an
    /// argument that matches no field, such as an option name, a subcommand name or an extra
    /// positional argument, since those are reported as text by the other variants.
    InvalidUtf8 {
        /// The argument name, e.g. `"--name <NAME>"`, or `None` for an argument that matches no
        /// field.
        name: Option<&'static str>,
        /// The raw bytes of the argument, displayed with [`Lossy`](crate::Lossy).
        value: &'a [u8],
    },

    /// Help was requested with `-h` or `--help`.
    ///
    /// This is not a parse failure: the string is the generated help text, the struct's `HELP`
    /// constant for `-h` and its `LONG_HELP` constant for `--help`. After a subcommand, it is the
    /// help of that subcommand instead. Applications should print it to standard output and exit
    /// successfully.
    /// It is never returned when help is disabled with `#[command(disable_help_flag)]`.
    Help(&'static str),

    /// The version was requested with `-V` or `--version`.
    ///
    /// This is not a parse failure: the string is the version line, the command name followed by
    /// the version (the struct's `VERSION_TEXT` constant). Applications should print it to standard
    /// output and exit successfully. It is only returned when `#[command(version)]` is set.
    Version(&'static str),
}

impl<'a> Error<'a> {
    /// Returns `true` if this is [`Error::Help`], i.e. the user asked for help rather than
    /// passing invalid arguments.
    #[must_use]
    pub fn is_help(&self) -> bool {
        matches!(self, Error::Help(_))
    }

    /// Returns `true` if this is [`Error::Version`], i.e. the user asked for the version rather
    /// than passing invalid arguments.
    #[must_use]
    pub fn is_version(&self) -> bool {
        matches!(self, Error::Version(_))
    }

    /// Pairs the error with the usage of the command, so that its `Display` ends with the usage
    /// line and a hint to try `--help`, as Clap errors do. See [`WithUsage`].
    ///
    /// The usage of a struct deriving `GetArgs` is its `USAGE` constant. For an error raised
    /// by a subcommand it is the usage of the struct holding the subcommand, not of the
    /// subcommand.
    ///
    /// ```
    /// # use getargs_derive::GetArgs;
    /// #[derive(Debug, GetArgs)]
    /// #[command(name = "copy")]
    /// struct Cli<'a> {
    ///     input: &'a str,
    /// }
    ///
    /// let error = Cli::from_args(["a", "b"].into_iter()).unwrap_err();
    /// assert_eq!(
    ///     error.with_usage(Cli::USAGE).to_string(),
    ///     "unexpected argument 'b' found\n\nUsage: copy [OPTIONS] <INPUT>\n\nFor more information, try '--help'."
    /// );
    /// ```
    pub const fn with_usage(self, usage: Usage) -> WithUsage<'a> {
        WithUsage { error: self, usage }
    }
}

impl core::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::MissingArgument(name) => {
                write!(
                    f,
                    "the following required argument was not provided: {name}"
                )
            }
            Error::UnknownOption { option, suggestion } => {
                write!(f, "unexpected argument '{option}' found")?;
                match suggestion {
                    Some(suggestion) => {
                        write!(f, "\n\n  tip: a similar argument exists: '--{suggestion}'")
                    }
                    None => Ok(()),
                }
            }
            Error::UnexpectedArgument(argument) => {
                write!(f, "unexpected argument '{argument}' found")
            }
            Error::UnexpectedValue(option) => {
                write!(
                    f,
                    "unexpected value for '{option}': it does not take a value"
                )
            }
            Error::ArgumentConflict { name, other } => {
                write!(f, "the argument '{name}' cannot be used with '{other}'")
            }
            Error::MissingRequiredArgument { name, required_by } => {
                write!(
                    f,
                    "the following required argument was not provided: {name} (required by '{required_by}')"
                )
            }
            Error::MissingOneOf(names) => {
                f.write_str("the following required arguments were not provided: <")?;
                for (index, name) in names.iter().enumerate() {
                    if index > 0 {
                        f.write_str("|")?;
                    }
                    f.write_str(name)?;
                }
                f.write_str(">")
            }
            Error::MissingSubcommand(names) => {
                f.write_str(
                    "a subcommand is required but one was not provided\n\n  [subcommands: ",
                )?;
                for (index, name) in names.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_str(name)?;
                }
                f.write_str("]")
            }
            Error::UnknownSubcommand { name, suggestion } => {
                write!(f, "unrecognized subcommand '{name}'")?;
                match suggestion {
                    Some(suggestion) => {
                        write!(f, "\n\n  tip: a similar subcommand exists: '{suggestion}'")
                    }
                    None => Ok(()),
                }
            }
            Error::InvalidArguments => f.write_str("the arguments could not be parsed"),
            Error::MissingOptionValue(name) => {
                write!(f, "a value is required for '{name}' but none was supplied")
            }
            Error::InvalidValue { name, value } => {
                write!(f, "invalid value '{value}' for '{name}'")
            }
            Error::InvalidPossibleValue {
                name,
                value,
                possible_values,
            } => {
                write!(
                    f,
                    "invalid value '{value}' for '{name}'\n  [possible values: "
                )?;
                for (index, possible) in possible_values.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    if possible.is_empty() || possible.contains(char::is_whitespace) {
                        write!(f, "{possible:?}")?;
                    } else {
                        f.write_str(possible)?;
                    }
                }
                f.write_str("]")
            }
            Error::ValueOutOfRange { name, value, range } => {
                write!(
                    f,
                    "invalid value '{value}' for '{name}': {value} is not in {range}"
                )
            }
            Error::TooManyValues { name, value, .. } => {
                write!(
                    f,
                    "unexpected value '{value}' for '{name}' found; no more were expected"
                )
            }
            Error::TooFewValues { name, min, actual } => {
                let verb = if *actual == 1 { "was" } else { "were" };
                write!(
                    f,
                    "{min} values required by '{name}'; only {actual} {verb} provided"
                )
            }
            Error::InvalidUtf8 {
                name: Some(name),
                value,
            } => {
                write!(
                    f,
                    "invalid UTF-8 in value '{}' for '{name}'",
                    crate::Lossy(value)
                )
            }
            Error::InvalidUtf8 { name: None, value } => {
                write!(f, "invalid UTF-8 in argument '{}'", crate::Lossy(value))
            }
            Error::Help(help) => f.write_str(help),
            Error::Version(version) => f.write_str(version),
        }
    }
}

impl core::error::Error for Error<'_> {}

/// The usage line of a command and whether it has a help flag, to remind the user of both after
/// an argument error, as Clap does.
///
/// The `GetArgs` derive generates one as the `USAGE` constant of the struct, built at compile
/// time without allocation. Its `Display` shows the usage line alone, e.g.
/// `Usage: copy [OPTIONS] <INPUT>`. Pass it to [`Error::with_usage`] to show it after an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Usage {
    /// The usage line, starting with `Usage: ` and without a trailing newline.
    pub line: &'static str,
    /// Whether the command has a help flag, which the hint after the usage line refers to.
    pub help_flag: bool,
}

impl Usage {
    /// A usage with the line `line` and a help flag.
    #[must_use]
    pub const fn new(line: &'static str) -> Self {
        Usage {
            line,
            help_flag: true,
        }
    }

    /// Sets [`help_flag`](Self::help_flag).
    #[must_use]
    pub const fn help_flag(mut self, help_flag: bool) -> Self {
        self.help_flag = help_flag;
        self
    }
}

impl core::fmt::Display for Usage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.line)
    }
}

/// An [`Error`] followed by a usage line and a help hint, returned by [`Error::with_usage`].
///
/// Its `Display` writes the message of the error, a blank line, the usage line, another blank
/// line and `For more information, try '--help'.` (left out when the command has no help flag),
/// like the end of a Clap error:
///
/// ```text
/// invalid value 'x' for '--mode <MODE>'
///   [possible values: fast, slow]
///
/// Usage: run [OPTIONS]
///
/// For more information, try '--help'.
/// ```
///
/// [`Error::Help`] and [`Error::Version`] are not failures and are written alone.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithUsage<'a> {
    error: Error<'a>,
    usage: Usage,
}

impl<'a> WithUsage<'a> {
    /// The error being reported.
    pub const fn error(&self) -> &Error<'a> {
        &self.error
    }

    /// The usage shown after the error.
    pub const fn usage(&self) -> &Usage {
        &self.usage
    }
}

impl core::fmt::Display for WithUsage<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.error)?;
        if self.error.is_help() || self.error.is_version() {
            return Ok(());
        }
        write!(f, "\n\n{}", self.usage)?;
        if self.usage.help_flag {
            f.write_str("\n\nFor more information, try '--help'.")?;
        }
        Ok(())
    }
}

impl core::error::Error for WithUsage<'_> {}
