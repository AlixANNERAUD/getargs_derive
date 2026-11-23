use getargs_derive::{GetArgs, Subcommand};

#[derive(GetArgs)]
struct Cli {
    #[arg(long, skip)]
    first: u8,
    #[arg(skip = 4)]
    #[arg(default_value_t = 4)]
    second: u8,
    #[arg(skip(4))]
    third: u8,
    #[arg(skip, skip = 1)]
    fourth: u8,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(short, skip)]
        jobs: u8,
    },
}

fn main() {}
