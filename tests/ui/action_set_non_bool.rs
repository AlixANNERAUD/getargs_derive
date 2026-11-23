use getargs_derive::GetArgs;

#[derive(GetArgs)]
struct Cli<'a> {
    #[arg(long, action = Set)]
    jobs: u8,
    #[arg(long, action = ArgAction::Set)]
    name: &'a str,
}

fn main() {}
