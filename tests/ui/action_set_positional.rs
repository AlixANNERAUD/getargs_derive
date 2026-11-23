use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli {
    #[arg(action = Set)]
    enabled: bool,
}

fn main() {}
