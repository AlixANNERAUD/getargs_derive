use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(long, action = Set, default_value = "yes")]
    enabled: bool,
}

fn main() {}
