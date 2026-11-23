use getargs_derive::{GetArgs, Subcommand};

#[derive(GetArgs)]
struct Inner {
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Run,
}

#[derive(GetArgs)]
struct Flattened {
    #[command(flatten)]
    #[arg(skip)]
    inner: Inner,
}

#[derive(GetArgs)]
struct WithSubcommand {
    #[arg(skip)]
    #[command(subcommand)]
    command: Commands,
}

fn main() {}
