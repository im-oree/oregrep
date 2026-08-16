use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScaffoldHookArgs {
    name: String,
    #[arg(short = 'o', long, default_value = "src/hooks")]
    out_dir: PathBuf,
}

pub fn run(args: ScaffoldHookArgs) -> Result<()> {
    let name = if args.name.starts_with("use") { args.name.clone() } else { format!("use{}", capitalize(&args.name)) };
    std::fs::create_dir_all(&args.out_dir)?;
    let f = args.out_dir.join(format!("{}.ts", name));
    let content = format!(
        "import {{ useState }} from 'react';\n\nexport function {}() {{\n  const [state, setState] = useState<unknown>(null);\n  return {{ state, setState }};\n}}\n", name);
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
