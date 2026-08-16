use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScaffoldAddArgs {
    /// Feature to add
    feature: AddFeature,

    /// Project dir (default: current)
    #[arg(default_value = ".")]
    dir: PathBuf,

    /// Package manager
    #[arg(long, default_value = "npm")]
    pm: String,

    #[arg(long)]
    no_install: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum AddFeature {
    Tailwind,
    Zustand,
    Router,
    Prettier,
    Eslint,
    Vitest,
    Jest,
    Playwright,
}

pub fn run(args: ScaffoldAddArgs) -> Result<()> {
    if !args.dir.exists() { anyhow::bail!("Dir not found: {}", args.dir.display()); }
    println!("{} {:?} to {}", "Adding:".cyan().bold(), args.feature, args.dir.display().to_string().dimmed());

    let (deps, dev_deps, files): (Vec<&str>, Vec<&str>, Vec<(&str, String)>) = match args.feature {
        AddFeature::Tailwind => (vec![], vec!["tailwindcss", "postcss", "autoprefixer"], vec![
            ("tailwind.config.js", "export default {\n  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],\n  theme: { extend: {} },\n  plugins: []\n};\n".to_string()),
            ("postcss.config.js", "export default { plugins: { tailwindcss: {}, autoprefixer: {} } };\n".to_string()),
        ]),
        AddFeature::Zustand => (vec!["zustand"], vec![], vec![]),
        AddFeature::Router => (vec!["react-router-dom"], vec![], vec![]),
        AddFeature::Prettier => (vec![], vec!["prettier"], vec![
            (".prettierrc", "{\n  \"semi\": true,\n  \"singleQuote\": true,\n  \"tabWidth\": 2,\n  \"trailingComma\": \"all\"\n}\n".to_string()),
        ]),
        AddFeature::Eslint => (vec![], vec!["eslint"], vec![
            (".eslintrc.json", "{\n  \"env\": { \"browser\": true, \"es2022\": true },\n  \"extends\": [\"eslint:recommended\"]\n}\n".to_string()),
        ]),
        AddFeature::Vitest => (vec![], vec!["vitest", "@vitest/ui"], vec![]),
        AddFeature::Jest => (vec![], vec!["jest", "@types/jest"], vec![
            ("jest.config.js", "export default { testEnvironment: 'node' };\n".to_string()),
        ]),
        AddFeature::Playwright => (vec![], vec!["@playwright/test"], vec![]),
    };

    for (rel, content) in &files {
        let p = args.dir.join(rel);
        std::fs::write(&p, content)?;
        println!("  {} {}", "+".green(), p.display().to_string().dimmed());
    }
    if !args.no_install && (!deps.is_empty() || !dev_deps.is_empty()) {
        if !deps.is_empty() {
            let mut c = std::process::Command::new(&args.pm);
            c.arg("install").args(&deps).current_dir(&args.dir);
            let _ = c.status();
        }
        if !dev_deps.is_empty() {
            let mut c = std::process::Command::new(&args.pm);
            c.arg("install").arg("-D").args(&dev_deps).current_dir(&args.dir);
            let _ = c.status();
        }
    }
    println!("\n{}", "Done.".green().bold());
    Ok(())
}
