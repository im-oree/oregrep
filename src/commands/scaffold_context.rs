use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScaffoldContextArgs {
    name: String,
    #[arg(short = 'o', long, default_value = "src/contexts")]
    out_dir: PathBuf,
}

pub fn run(args: ScaffoldContextArgs) -> Result<()> {
    std::fs::create_dir_all(&args.out_dir)?;
    let f = args.out_dir.join(format!("{}-context.tsx", args.name.to_lowercase()));
    let name = capitalize(&args.name);
    let content = format!(
        "import React, {{ createContext, useContext, useState }} from 'react';\n\ninterface {n}ContextValue {{\n  value: string;\n  setValue: (v: string) => void;\n}}\n\nconst {n}Context = createContext<{n}ContextValue | null>(null);\n\nexport function {n}Provider({{ children }}: {{ children: React.ReactNode }}) {{\n  const [value, setValue] = useState('');\n  return <{n}Context.Provider value={{{{ value, setValue }}}}>{{children}}</{n}Context.Provider>;\n}}\n\nexport function use{n}() {{\n  const ctx = useContext({n}Context);\n  if (!ctx) throw new Error('use{n} must be inside {n}Provider');\n  return ctx;\n}}\n", n = name);
    std::fs::write(&f, content)?;
    println!("  {} {}", "+".green(), f.display().to_string().dimmed());
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
