use anyhow::{anyhow, Result};
use clap::Parser;
use regex::{Regex, RegexBuilder};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(value_name = "FILE", help = "Input files or directories")]
    sources: Vec<String>,

    #[arg(short = 'm', long, help = "Pattern")]
    pattern: Option<String>,

    #[arg(short, long, help = "Case-insensitive pattern matching")]
    insensitive: bool,

    #[arg(short, long, help = "Random seed")]
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> Result<Config> {
    let args = Args::parse();
    let pattern = args
        .pattern
        .map(|val| {
            RegexBuilder::new(&val)
                .case_insensitive(args.insensitive)
                .build()
                .map_err(|_| anyhow!("Invalid --pattern \"{val}\""))
        })
        .transpose()?;
    Ok(Config {
        sources: args.sources,
        pattern,
        seed: args.seed,
    })
}

pub fn run(config: Config) -> Result<()> {
    dbg!(config);
    Ok(())
}
