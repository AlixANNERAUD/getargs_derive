# Flag Actions

Like Clap, a flag's behaviour can be chosen with `action`. Only this subset of Clap's `ArgAction` is supported, written as a bare name or as `ArgAction::Name`:
the three flag actions below, and `Set`, which turns a `bool` field into an option taking a value (see [Boolean Options](#boolean-options)).

| Action     | Field type                            | Absent         | Each occurrence                       |
| ---------- | ------------------------------------- | -------------- | ------------------------------------- |
| `SetTrue`  | `bool`                                | `false`        | sets `true` (the default for `bool`)  |
| `SetFalse` | `bool`                                | `true`         | sets `false`                          |
| `Count`    | integer (`u8`, `u32`, `i64`, `usize`, ...) | `0`       | adds one, saturating at the type's maximum |

`Count` counts every occurrence, including repeated short flags grouped in one argument (`-vvv`) and mixed short and long forms (`-vv --verbose`).
A default (`default_value_t = ...`) replaces the value used when the flag is absent; as in Clap, it is not a starting point for counting.
Flags never take a value, so `--verbose=2` is rejected as an unknown option.

```rust
use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    /// More output, repeat for even more.
    #[arg(short, long, action = Count)]
    verbose: u8,
    /// Disable colored output.
    #[arg(long = "no-color", action = SetFalse)]
    color: bool,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let cli = Cli::from_args(["-vvv", "--verbose", "--no-color"].into_iter())?;
assert_eq!(cli.verbose, 4);
assert!(!cli.color);

let cli = Cli::from_args([].into_iter())?;
assert_eq!(cli.verbose, 0);
assert!(cli.color);
# Ok(())
# }
```

In the generated help, a `Count` flag is followed by `...` (`-v, --verbose...`), as in Clap. See [Generated Help][guide-help].

## Boolean Options

A named `bool` field is a flag unless it has `action = Set`, Clap's `ArgAction::Set`. It is then an ordinary option taking a value,
`--dnssd true`, `--dnssd=false` or `-d false`, parsed with `bool::from_str`, so only `true` and `false` are accepted.
Like any other option it is required unless it has a default (`default_value` or `default_value_t`), and it supports
[`env`][guide-env], `value_name`, `value_parser` functions and `help`. As in Clap, its help lists `[possible values: true, false]`, and a
value that is neither is reported as `Error::InvalidPossibleValue`. A `value_parser` function replaces this check, and then accepts the values it parses.

```rust
use getargs_derive::GetArgs;

#[derive(Debug, GetArgs)]
#[command(name = "serve")]
struct Cli {
    /// Advertise the service with DNS-SD.
    #[arg(long, action = Set, default_value_t = true)]
    dnssd: bool,
    /// Use the cache.
    #[arg(long, action = Set, env = "SERVE_CACHE")]
    cache: bool,
}

# fn main() -> Result<(), getargs_derive::Error<'static>> {
let cli = Cli::from_args_with_env(["--dnssd", "false"].into_iter(), |_| Some("true"))?;
assert!(!cli.dnssd);
assert!(cli.cache);

let error = Cli::from_args_with_env(["--cache=yes"].into_iter(), |_| None).unwrap_err();
assert_eq!(
    error.to_string(),
    "invalid value 'yes' for '--cache <CACHE>'\n  [possible values: true, false]"
);

assert_eq!(
    Cli::HELP,
    "Usage: serve [OPTIONS]

Options:
      --dnssd <DNSSD>  Advertise the service with DNS-SD [default: true] [possible values: true, false]
      --cache <CACHE>  Use the cache [env: SERVE_CACHE=] [possible values: true, false]
  -h, --help           Print help"
);
# Ok(())
# }
```

## Compile-Time Diagnostics

The macro reports a compile error, pointing at the offending attribute or field, for:

- unsupported `action` values, and `action` without a value;
- `action` without `short`/`long`, including `Set`: a boolean positional argument is not supported;
- `SetTrue`, `SetFalse` or `Set` on a non-`bool` field, and `Count` on a non-integer field;
- `value_name` on a field with a flag action;
- a `default_value` other than `true` or `false` on a `Set` field without a `value_parser`.

[guide-env]: env.md
[guide-help]: help.md
