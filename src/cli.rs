use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct CompileArgs {
    #[arg(required = true)]
    pub input: Vec<PathBuf>,

    #[arg(long, default_value = "gcc")]
    pub cc: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub enum Args {
    #[command(version, about, long_about = "Compiles a program")]
    C(CompileArgs),
}
