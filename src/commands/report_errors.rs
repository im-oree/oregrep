use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::compile::load_report;

use crate::commands::report_health::write_out;

#[derive(Args)]
pub struct ReportErrorsArgs {
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: ReportErrorsArgs) -> Result<()> {
    let report = match load_report()? {
        Some(r) => r,
        None => { println!("No cached compile report. Run compile-ts or compile-rust first."); return Ok(()); }
    };
    let mut md = String::new();
    md.push_str("# Compile Errors Report\n\n");
    md.push_str(&format!("**Tool:** {}  \n**Timestamp:** {}  \n**Exit code:** {}\n\n",
        report.tool, report.timestamp, report.exit_code));
    md.push_str(&format!("**Errors:** {}  \n**Warnings:** {}\n\n",
        report.errors.len(), report.warnings.len()));

    let mut by_file: HashMap<String, (Vec<_>, Vec<_>)> = HashMap::new();
    for e in &report.errors { by_file.entry(e.file.clone()).or_default().0.push(e.clone()); }
    for w in &report.warnings { by_file.entry(w.file.clone()).or_default().1.push(w.clone()); }
    let mut files: Vec<&String> = by_file.keys().collect();
    files.sort();

    for f in files {
        let (es, ws) = &by_file[f];
        md.push_str(&format!("## `{}`\n\n", f));
        if !es.is_empty() {
            md.push_str("### Errors\n\n");
            for e in es {
                md.push_str(&format!("- L{}:{}  `{}` — {}\n", e.line, e.column, e.code, e.message));
            }
            md.push('\n');
        }
        if !ws.is_empty() {
            md.push_str("### Warnings\n\n");
            for w in ws {
                md.push_str(&format!("- L{}:{}  `{}` — {}\n", w.line, w.column, w.code, w.message));
            }
            md.push('\n');
        }
    }
    write_out(&md, args.output)
}
