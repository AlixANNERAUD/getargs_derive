use getargs_derive::GetArgs;

struct NoDefault;

#[derive(GetArgs)]
struct Cli {
    #[arg(skip)]
    value: NoDefault,
}

fn main() {}
