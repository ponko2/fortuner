use anyhow::{anyhow, bail, Result};
use clap::Parser;
use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};
use regex::{Regex, RegexBuilder};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
    vec,
};
use walkdir::WalkDir;

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
    match config.pattern {
        Some(pattern) => {
            let mut prev_source = None;
            fortunes
                .iter()
                .filter(|fortune| pattern.is_match(&fortune.text))
                .for_each(|fortune| {
                    prev_source
                        .as_ref()
                        .map_or(true, |source| source != &fortune.source)
                        .then(|| {
                            eprintln!("({})\n%", fortune.source);
                            prev_source = Some(fortune.source.clone());
                        });
                    println!("{}\n%", fortune.text);
                });
        }
        None => {
            println!(
                "{}",
                pick_fortune(&fortunes, config.seed)
                    .unwrap_or_else(|| "No fortunes found".to_string())
            );
        }
    }
    Ok(())
}

fn find_files(paths: &[String]) -> Result<Vec<PathBuf>> {
    let dat = OsStr::new("dat");
    let mut files = vec![];
    for path in paths {
        match fs::metadata(path) {
            Err(err) => bail!("{path}: {err}"),
            Ok(_) => files.extend(
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|entry| {
                        entry.file_type().is_file() && entry.path().extension() != Some(dat)
                    })
                    .map(|entry| entry.path().into()),
            ),
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn read_fortunes(paths: &[PathBuf]) -> Result<Vec<Fortune>> {
    let mut fortunes = vec![];
    let mut buf = vec![];
    for path in paths {
        let basename = path.file_name().unwrap().to_string_lossy().into_owned();
        let file = File::open(path)
            .map_err(|err| anyhow!("{}: {err}", path.to_string_lossy().into_owned()))?;
        BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .for_each(|line| {
                if line == "%" {
                    if !buf.is_empty() {
                        fortunes.push(Fortune {
                            source: basename.clone(),
                            text: buf.join("\n"),
                        });
                        buf.clear();
                    }
                } else {
                    buf.push(line.to_string());
                }
            });
    }
    Ok(fortunes)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    let mut rng: Box<dyn RngCore> = match seed {
        Some(state) => Box::new(StdRng::seed_from_u64(state)),
        None => Box::new(rand::thread_rng()),
    };
    fortunes
        .choose(&mut rng)
        .map(|fortune| fortune.text.to_string())
}

#[cfg(test)]
mod tests {
    use super::{find_files, pick_fortune, read_fortunes, Fortune};
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

    #[test]
    fn test_pick_fortune() {
        // Fortuneのスライスを作成
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // シードを与えて引用句を1つ選択
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
