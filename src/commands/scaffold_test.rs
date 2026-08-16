use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScaffoldTestArgs {
    /// Existing file to create a test for
    file: PathBuf,

    /// Test framework
    #[arg(short = 'f', long, default_value = "vitest")]
    framework: String,
}

pub fn run(args: ScaffoldTestArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let stem = args.file.file_stem().and_then(|s| s.to_str()).unwrap_or("test");
    let ext = args.file.extension().and_then(|e| e.to_str()).unwrap_or("ts");
    let test_path = args.file.parent().unwrap_or(std::path::Path::new(".")).join(format!("{}.test.{}", stem, ext));
    if test_path.exists() { anyhow::bail!("Test file exists: {}", test_path.display()); }

    let import = format!("./{}", stem);
    let content = match args.framework.as_str() {
        "jest" => format!(
            "import * as mod from '{}';\n\ndescribe('{}', () => {{\n  it('exports something', () => {{\n    expect(mod).toBeDefined();\n  }});\n}});\n", import, stem),
        _ => format!(
            "import {{ describe, it, expect }} from 'vitest';\nimport * as mod from '{}';\n\ndescribe('{}', () => {{\n  it('exports something', () => {{\n    expect(mod).toBeDefined();\n  }});\n}});\n", import, stem),
    };
    std::fs::write(&test_path, content)?;
    println!("  {} {}", "+".green(), test_path.display().to_string().dimmed());
    Ok(())
}
