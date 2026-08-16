use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScaffoldArgs {
    /// Project template to scaffold
    template: Template,

    /// Project name / directory
    name: String,

    /// Parent directory (default: current dir)
    #[arg(short = 'p', long, default_value = ".")]
    parent: PathBuf,

    /// Package manager for JS templates (npm | yarn | pnpm)
    #[arg(long, default_value = "npm")]
    pm: String,

    /// Skip installing dependencies
    #[arg(long)]
    no_install: bool,

    /// Skip git init
    #[arg(long)]
    no_git: bool,

    #[arg(long)]
    dry_run: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Template {
    ReactApp,
    NextApp,
    ViteApp,
    ElectronApp,
    RustCli,
    RustLib,
    NodeApp,
    NodeLib,
    TypescriptLib,
    PythonApp,
    Monorepo,
    Static,
}

pub fn run(args: ScaffoldArgs) -> Result<()> {
    let target = args.parent.join(&args.name);
    if target.exists() {
        anyhow::bail!("Target already exists: {}", target.display());
    }
    println!("{} {} at {}",
        "Scaffolding:".cyan().bold(),
        format!("{:?}", args.template).yellow(),
        target.display().to_string().cyan());
    if args.dry_run { println!("{}", "[DRY RUN — would create]".yellow().bold()); return Ok(()); }

    std::fs::create_dir_all(&target)?;
    match args.template {
        Template::RustCli => scaffold_rust_cli(&target, &args.name)?,
        Template::RustLib => scaffold_rust_lib(&target, &args.name)?,
        Template::ReactApp => scaffold_react(&target, &args.name, &args.pm, args.no_install)?,
        Template::NextApp => scaffold_next(&target, &args.name, &args.pm, args.no_install)?,
        Template::ViteApp => scaffold_vite(&target, &args.name, &args.pm, args.no_install)?,
        Template::ElectronApp => scaffold_electron(&target, &args.name, &args.pm, args.no_install)?,
        Template::NodeApp => scaffold_node(&target, &args.name)?,
        Template::NodeLib => scaffold_node_lib(&target, &args.name)?,
        Template::TypescriptLib => scaffold_ts_lib(&target, &args.name)?,
        Template::PythonApp => scaffold_python(&target, &args.name)?,
        Template::Monorepo => scaffold_monorepo(&target, &args.name)?,
        Template::Static => scaffold_static(&target, &args.name)?,
    }

    if !args.no_git {
        let _ = std::process::Command::new("git").arg("init").current_dir(&target).status();
        println!("  {} git initialized", "→".dimmed());
    }
    println!("\n{} {}", "Done:".green().bold(), target.display().to_string().cyan());
    println!("  cd {}", args.name.dimmed());
    Ok(())
}

fn write(path: &std::path::Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    std::fs::write(path, content)?;
    println!("  {} {}", "+".green(), path.display().to_string().dimmed());
    Ok(())
}

fn scaffold_rust_cli(dir: &std::path::Path, name: &str) -> Result<()> {
    write(&dir.join("Cargo.toml"), &format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nclap = {{ version = \"4\", features = [\"derive\"] }}\nanyhow = \"1\"\n", name))?;
    write(&dir.join("src/main.rs"),
        "use anyhow::Result;\nuse clap::Parser;\n\n#[derive(Parser)]\nstruct Cli {\n    /// Name to greet\n    #[arg(short, long, default_value = \"world\")]\n    name: String,\n}\n\nfn main() -> Result<()> {\n    let cli = Cli::parse();\n    println!(\"Hello, {}!\", cli.name);\n    Ok(())\n}\n")?;
    write(&dir.join(".gitignore"), "target/\nCargo.lock\n")?;
    write(&dir.join("README.md"), &format!("# {}\n\nA Rust CLI application.\n\n## Build\n\n```\ncargo build --release\n```\n", name))?;
    Ok(())
}

fn scaffold_rust_lib(dir: &std::path::Path, name: &str) -> Result<()> {
    write(&dir.join("Cargo.toml"), &format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\n\n[dependencies]\n", name))?;
    write(&dir.join("src/lib.rs"),
        "pub fn add(a: i64, b: i64) -> i64 { a + b }\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn it_works() { assert_eq!(add(2, 2), 4); }\n}\n")?;
    write(&dir.join(".gitignore"), "target/\nCargo.lock\n")?;
    write(&dir.join("README.md"), &format!("# {}\n\nA Rust library.\n", name))?;
    Ok(())
}

fn scaffold_react(dir: &std::path::Path, name: &str, pm: &str, no_install: bool) -> Result<()> {
    write(&dir.join("package.json"), &format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  }},
  "devDependencies": {{
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  }}
}}
"#, name))?;
    write(&dir.join("index.html"),
        "<!DOCTYPE html>\n<html>\n<head><meta charset=\"UTF-8\"><title>App</title></head>\n<body><div id=\"root\"></div><script type=\"module\" src=\"/src/main.tsx\"></script></body>\n</html>\n")?;
    write(&dir.join("vite.config.ts"),
        "import { defineConfig } from 'vite';\nimport react from '@vitejs/plugin-react';\nexport default defineConfig({ plugins: [react()] });\n")?;
    write(&dir.join("tsconfig.json"), "{\n  \"compilerOptions\": {\n    \"target\": \"ES2020\",\n    \"lib\": [\"ES2020\", \"DOM\"],\n    \"jsx\": \"react-jsx\",\n    \"module\": \"ESNext\",\n    \"moduleResolution\": \"bundler\",\n    \"strict\": true,\n    \"skipLibCheck\": true\n  },\n  \"include\": [\"src\"]\n}\n")?;
    write(&dir.join("src/main.tsx"),
        "import React from 'react';\nimport ReactDOM from 'react-dom/client';\nimport { App } from './App';\n\nReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode><App /></React.StrictMode>);\n")?;
    write(&dir.join("src/App.tsx"),
        &format!("export function App() {{\n  return <div><h1>{}</h1></div>;\n}}\n", name))?;
    write(&dir.join(".gitignore"), "node_modules/\ndist/\n.vite/\n")?;
    if !no_install { install(pm, dir); }
    Ok(())
}

fn scaffold_next(dir: &std::path::Path, name: &str, pm: &str, no_install: bool) -> Result<()> {
    write(&dir.join("package.json"), &format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "dev": "next dev",
    "build": "next build",
    "start": "next start"
  }},
  "dependencies": {{
    "next": "^14.2.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  }},
  "devDependencies": {{
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@types/node": "^20.0.0",
    "typescript": "^5.5.0"
  }}
}}
"#, name))?;
    write(&dir.join("tsconfig.json"), "{\n  \"compilerOptions\": {\n    \"target\": \"ES2020\",\n    \"lib\": [\"dom\", \"dom.iterable\", \"esnext\"],\n    \"allowJs\": true,\n    \"skipLibCheck\": true,\n    \"strict\": true,\n    \"forceConsistentCasingInFileNames\": true,\n    \"noEmit\": true,\n    \"esModuleInterop\": true,\n    \"module\": \"esnext\",\n    \"moduleResolution\": \"bundler\",\n    \"resolveJsonModule\": true,\n    \"isolatedModules\": true,\n    \"jsx\": \"preserve\",\n    \"incremental\": true\n  },\n  \"include\": [\"next-env.d.ts\", \"**/*.ts\", \"**/*.tsx\"],\n  \"exclude\": [\"node_modules\"]\n}\n")?;
    write(&dir.join("app/page.tsx"), &format!("export default function Page() {{ return <h1>{}</h1>; }}\n", name))?;
    write(&dir.join("app/layout.tsx"), "export default function Layout({ children }: { children: React.ReactNode }) {\n  return <html><body>{children}</body></html>;\n}\n")?;
    write(&dir.join(".gitignore"), "node_modules/\n.next/\nout/\n")?;
    if !no_install { install(pm, dir); }
    Ok(())
}

fn scaffold_vite(dir: &std::path::Path, name: &str, pm: &str, no_install: bool) -> Result<()> {
    scaffold_react(dir, name, pm, no_install)
}

fn scaffold_electron(dir: &std::path::Path, name: &str, pm: &str, no_install: bool) -> Result<()> {
    write(&dir.join("package.json"), &format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "main": "main.js",
  "scripts": {{
    "start": "electron ."
  }},
  "devDependencies": {{
    "electron": "^32.0.0"
  }}
}}
"#, name))?;
    write(&dir.join("main.js"),
        "const { app, BrowserWindow } = require('electron');\nfunction createWindow() {\n  const w = new BrowserWindow({ width: 900, height: 700 });\n  w.loadFile('index.html');\n}\napp.whenReady().then(createWindow);\napp.on('window-all-closed', () => { if (process.platform !== 'darwin') app.quit(); });\n")?;
    write(&dir.join("index.html"), &format!("<!DOCTYPE html><html><body><h1>{}</h1></body></html>\n", name))?;
    write(&dir.join(".gitignore"), "node_modules/\ndist/\n")?;
    if !no_install { install(pm, dir); }
    Ok(())
}

fn scaffold_node(dir: &std::path::Path, name: &str) -> Result<()> {
    write(&dir.join("package.json"), &format!("{{\n  \"name\": \"{}\",\n  \"version\": \"0.1.0\",\n  \"main\": \"index.js\",\n  \"scripts\": {{ \"start\": \"node index.js\" }}\n}}\n", name))?;
    write(&dir.join("index.js"), &format!("console.log('Hello from {}');\n", name))?;
    write(&dir.join(".gitignore"), "node_modules/\n")?;
    Ok(())
}

fn scaffold_node_lib(dir: &std::path::Path, name: &str) -> Result<()> {
    write(&dir.join("package.json"), &format!(
        "{{\n  \"name\": \"{}\",\n  \"version\": \"0.1.0\",\n  \"main\": \"index.js\",\n  \"exports\": \"./index.js\"\n}}\n", name))?;
    write(&dir.join("index.js"), "export function add(a, b) { return a + b; }\n")?;
    write(&dir.join(".gitignore"), "node_modules/\n")?;
    Ok(())
}

fn scaffold_ts_lib(dir: &std::path::Path, name: &str) -> Result<()> {
    write(&dir.join("package.json"), &format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {{
    "build": "tsc"
  }},
  "devDependencies": {{
    "typescript": "^5.5.0"
  }}
}}
"#, name))?;
    write(&dir.join("tsconfig.json"), "{\n  \"compilerOptions\": {\n    \"target\": \"ES2020\",\n    \"module\": \"ESNext\",\n    \"declaration\": true,\n    \"outDir\": \"dist\",\n    \"strict\": true\n  },\n  \"include\": [\"src\"]\n}\n")?;
    write(&dir.join("src/index.ts"), "export function add(a: number, b: number): number { return a + b; }\n")?;
    write(&dir.join(".gitignore"), "node_modules/\ndist/\n")?;
    Ok(())
}

fn scaffold_python(dir: &std::path::Path, name: &str) -> Result<()> {
    write(&dir.join("pyproject.toml"), &format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0\"\ndescription = \"\"\nrequires-python = \">=3.10\"\n", name))?;
    write(&dir.join(format!("{}/__init__.py", name)), "__version__ = '0.1.0'\n")?;
    write(&dir.join(format!("{}/main.py", name)), "def main():\n    print('Hello')\n\nif __name__ == '__main__':\n    main()\n")?;
    write(&dir.join(".gitignore"), "__pycache__/\n*.pyc\n.venv/\ndist/\nbuild/\n*.egg-info/\n")?;
    Ok(())
}

fn scaffold_monorepo(dir: &std::path::Path, name: &str) -> Result<()> {
    write(&dir.join("package.json"), &format!(
        "{{\n  \"name\": \"{}\",\n  \"private\": true,\n  \"workspaces\": [\"packages/*\", \"apps/*\"]\n}}\n", name))?;
    write(&dir.join("packages/.keep"), "")?;
    write(&dir.join("apps/.keep"), "")?;
    write(&dir.join(".gitignore"), "node_modules/\ndist/\n")?;
    write(&dir.join("README.md"), &format!("# {}\n\nMonorepo.\n", name))?;
    Ok(())
}

fn scaffold_static(dir: &std::path::Path, name: &str) -> Result<()> {
    write(&dir.join("index.html"), &format!("<!DOCTYPE html><html><head><title>{}</title></head><body><h1>{}</h1></body></html>\n", name, name))?;
    write(&dir.join("styles.css"), "body { font-family: sans-serif; }\n")?;
    write(&dir.join(".gitignore"), "node_modules/\n")?;
    Ok(())
}

fn install(pm: &str, dir: &std::path::Path) {
    println!("  {} running {} install...", "→".dimmed(), pm.dimmed());
    let _ = std::process::Command::new(pm).arg("install").current_dir(dir).status();
}
