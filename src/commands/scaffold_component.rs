use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScaffoldComponentArgs {
    name: String,
    #[arg(short = 'o', long, default_value = "src/components")]
    out_dir: PathBuf,

    /// Include CSS module
    #[arg(long)]
    with_css: bool,

    /// Include test file
    #[arg(long)]
    with_test: bool,
}

pub fn run(args: ScaffoldComponentArgs) -> Result<()> {
    std::fs::create_dir_all(&args.out_dir)?;
    let tsx = args.out_dir.join(format!("{}.tsx", args.name));
    let content = format!(
        "import React from 'react';\n\n{}export interface {}Props {{\n  children?: React.ReactNode;\n}}\n\nexport function {}({{ children }}: {}Props) {{\n  return <div>{{children}}</div>;\n}}\n",
        if args.with_css { format!("import styles from './{}.module.css';\n\n", args.name) } else { String::new() },
        args.name, args.name, args.name);
    std::fs::write(&tsx, content)?;
    println!("  {} {}", "+".green(), tsx.display().to_string().dimmed());
    if args.with_css {
        let css = args.out_dir.join(format!("{}.module.css", args.name));
        std::fs::write(&css, ".root { }\n")?;
        println!("  {} {}", "+".green(), css.display().to_string().dimmed());
    }
    if args.with_test {
        let test = args.out_dir.join(format!("{}.test.tsx", args.name));
        std::fs::write(&test, format!("import {{ describe, it, expect }} from 'vitest';\nimport {{ {} }} from './{}';\n\ndescribe('{}', () => {{\n  it('renders', () => {{ /* ... */ }});\n}});\n", args.name, args.name, args.name))?;
        println!("  {} {}", "+".green(), test.display().to_string().dimmed());
    }
    println!("\n{}", "Done.".green().bold());
    Ok(())
}
