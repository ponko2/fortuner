use anyhow::{anyhow, Result};
use clap::Parser;
use regex::{Regex, RegexBuilder};
use std::path::PathBuf;

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

#[derive(Debug)]
pub struct Fortune {
    source: String,
    text: String,
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
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    dbg!(fortunes.last());
    Ok(())
}

fn find_files(paths: &[String]) -> Result<Vec<PathBuf>> {
    todo!()
}

fn read_fortunes(paths: &[PathBuf]) -> Result<Vec<Fortune>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{find_files, read_fortunes};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // 存在するファイルを検索できることを確認する
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.first().unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // 存在しないファイルの検索には失敗する
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // 拡張子が「.dat」以外の入力ファイルをすべて検索する
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // ファイル数とファイルの順番を確認する
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.first().unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // 複数のソースに対するテストをする。
        // パスは重複なしでソートされた状態でなければならない
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // 入力ファイルが1つだけの場合
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // 数が正しいこととソートされていることを確認する
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // 入力ファイルが複数の場合
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }
}
