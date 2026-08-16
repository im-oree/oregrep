use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct EnvGetArgs {
    file: PathBuf,
    /// Key (omit to list all)
    key: Option<String>,
}

pub fn run(args: EnvGetArgs) -> Result<()> {
    let map = load_env(&args.file)?;
    match args.key {
        Some(k) => match map.get(&k) {
            Some(v) => println!("{}", v),
            None => { eprintln!("Key not found: {}", k); std::process::exit(1); }
        },
        None => {
            for (k, v) in &map { println!("{}={}", k, v); }
        }
    }
    Ok(())
}

fn load_env(path: &std::path::Path) -> Result<std::collections::BTreeMap<String, String>> {
    let iter = dotenvy::from_path_iter(path)?;
    let mut m = std::collections::BTreeMap::new();
    for r in iter {
        let (k, v) = r?;
        m.insert(k, v);
    }
    Ok(m)
}
