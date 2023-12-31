use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[arg(short, long)]
    two: Option<String>,
    #[arg(short, long, value_name = "FILE")]
    one: PathBuf,
}

// This declaration will look for a file named `my.rs` and will
// insert its contents inside a module named `my` under this scope
mod my;

fn function() {
    println!("called `function()`");
}

fn main() {
    let cli = Cli::parse();

    if let Some(two) = cli.two.as_deref() {
        println!("Value for two: {two}");
    }

    println!("two: {:?}", cli.two);
    println!("one: {:?}", cli.one);

    my::function();

    function();

    my::indirect_access();

    my::nested::function();
}
