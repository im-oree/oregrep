use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScaffoldStoreArgs {
    name: String,
    #[arg(short = 'o', long, default_value = "src/store")]
    out_dir: PathBuf,
}

pub fn run(args: ScaffoldStoreArgs) -> Result<()> {
    std::fs::create_dir_all(&args.out_dir)?;
    let f = args.out_dir.join(format!("{}.ts", args.name.to_lowercase()));
    let store_name = format!("use{}Store", capitalize(&args.name));
    let content = format!(
        "import {{ create }} from 'zustand';\n\ninterface {name}State {{\n  count: number;\n  increment: () => void;\n  reset: () => void;\n}}\n\nexport const {store} = create<{name}State>((set) => ({{\n  count: 0,\n  increment: () => set((s) => ({{ count: s.count + 1 }})),\n  reset: () => set({{ count: 0 }}),\n}}));\n",
        name = capitalize(&args.name), store = store_name);
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
