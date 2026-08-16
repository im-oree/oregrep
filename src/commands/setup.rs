use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct SetupArgs {
    tool: SetupTool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SetupTool {
    Rust,
    Node,
    Git,
    Python,
    Env,
}

pub fn run(args: SetupArgs) -> Result<()> {
    println!("{} {:?}", "Setup check:".cyan().bold(), args.tool);
    match args.tool {
        SetupTool::Rust => check("rustc --version")?,
        SetupTool::Node => { check("node --version")?; check("npm --version")?; }
        SetupTool::Git => check("git --version")?,
        SetupTool::Python => check("python --version")?,
        SetupTool::Env => {
            check("rustc --version").ok();
            check("cargo --version").ok();
            check("node --version").ok();
            check("npm --version").ok();
            check("yarn --version").ok();
            check("pnpm --version").ok();
            check("python --version").ok();
            check("git --version").ok();
            check("code --version").ok();
        }
    }
    Ok(())
}

fn check(cmd: &str) -> Result<()> {
    let r = run_cmd(cmd, false, true)?;
    if r.success() {
        println!("  {} {} → {}", "OK".green().bold(), cmd.dimmed(), r.stdout.lines().next().unwrap_or("").trim());
    } else {
        println!("  {} {} — {}", "MISSING".red().bold(), cmd.dimmed(), "install this".yellow());
    }
    Ok(())
}
