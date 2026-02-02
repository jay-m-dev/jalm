use clap::{Parser, Subcommand};
use jalm_effectcheck::check as check_effects;
use jalm_formatter::format_source;
use jalm_parser::parse;
use jalm_typecheck::check;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "jalmt", version, about = "JaLM toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Parse { file: PathBuf },
    Fmt { file: PathBuf },
    Check { file: PathBuf },
    New { name: String, #[arg(long)] dir: Option<PathBuf> },
    Build { #[arg(long)] dir: Option<PathBuf> },
    Test { #[arg(long)] dir: Option<PathBuf> },
    Run { #[arg(long)] dir: Option<PathBuf> },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Parse { file } => cmd_parse(&file),
        Command::Fmt { file } => cmd_fmt(&file),
        Command::Check { file } => cmd_check(&file),
        Command::New { name, dir } => cmd_new(&name, dir.as_deref()),
        Command::Build { dir } => cmd_build(dir.as_deref()),
        Command::Test { dir } => cmd_test(dir.as_deref()),
        Command::Run { dir } => cmd_run(dir.as_deref()),
    };

    if let Err(err) = result {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn cmd_parse(path: &Path) -> Result<(), String> {
    let source = read_file(path)?;
    let parsed = parse(&source);
    let diag = json!({
        "errors": parsed.errors,
    });
    println!("{}", serde_json::to_string_pretty(&diag).unwrap());
    Ok(())
}

fn cmd_fmt(path: &Path) -> Result<(), String> {
    let source = read_file(path)?;
    match format_source(&source) {
        Ok(formatted) => {
            if formatted != source {
                fs::write(path, formatted).map_err(|e| format!("failed to write {}: {e}", path.display()))?;
            }
            Ok(())
        }
        Err(err) => Err(format!("format error: {err:?}")),
    }
}

fn cmd_check(path: &Path) -> Result<(), String> {
    let source = read_file(path)?;
    let tc = check(&source);
    let ec = check_effects(&source);
    let diag = json!({
        "type_diagnostics": tc.diagnostics,
        "effect_diagnostics": ec.diagnostics,
    });
    println!("{}", serde_json::to_string_pretty(&diag).unwrap());
    Ok(())
}

fn cmd_new(name: &str, dir: Option<&Path>) -> Result<(), String> {
    let root = dir.unwrap_or_else(|| Path::new("."));
    let project_dir = root.join(name);
    if project_dir.exists() {
        return Err(format!("destination {} already exists", project_dir.display()));
    }

    fs::create_dir_all(project_dir.join("src")).map_err(|e| format!("create project: {e}"))?;
    fs::create_dir_all(project_dir.join("tests")).map_err(|e| format!("create tests: {e}"))?;

    fs::write(
        project_dir.join("jalm.toml"),
        format!("name = \"{}\"\nversion = \"0.1.0\"\n", name),
    )
    .map_err(|e| format!("write jalm.toml: {e}"))?;

    fs::write(
        project_dir.join("jalm.lock"),
        "# JaLM lockfile (v0)\n# Deterministic builds placeholder\n",
    )
    .map_err(|e| format!("write jalm.lock: {e}"))?;

    fs::write(
        project_dir.join("src/main.jalm"),
        "fn main() -> i64 {\n  return 0;\n}\n",
    )
    .map_err(|e| format!("write src/main.jalm: {e}"))?;

    fs::write(
        project_dir.join("tests/basic.jalm"),
        "fn add(a: i64, b: i64) -> i64 {\n  return a + b;\n}\n",
    )
    .map_err(|e| format!("write tests/basic.jalm: {e}"))?;

    Ok(())
}

fn cmd_build(dir: Option<&Path>) -> Result<(), String> {
    let root = dir.unwrap_or_else(|| Path::new("."));
    let source = read_file(&root.join("src/main.jalm"))?;
    let parsed = parse(&source);
    if !parsed.errors.is_empty() {
        return Err("parse errors in src/main.jalm".to_string());
    }
    let tc = check(&source);
    let ec = check_effects(&source);
    if !tc.diagnostics.is_empty() || !ec.diagnostics.is_empty() {
        return Err("check failed for src/main.jalm".to_string());
    }
    Ok(())
}

fn cmd_test(dir: Option<&Path>) -> Result<(), String> {
    let root = dir.unwrap_or_else(|| Path::new("."));
    let entries = fs::read_dir(root.join("tests")).map_err(|e| format!("read tests: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("jalm") {
            continue;
        }
        let source = read_file(&path)?;
        let parsed = parse(&source);
        if !parsed.errors.is_empty() {
            return Err(format!("parse errors in {}", path.display()));
        }
        let tc = check(&source);
        let ec = check_effects(&source);
        if !tc.diagnostics.is_empty() || !ec.diagnostics.is_empty() {
            return Err(format!("check failed for {}", path.display()));
        }
    }
    Ok(())
}

fn cmd_run(dir: Option<&Path>) -> Result<(), String> {
    let root = dir.unwrap_or_else(|| Path::new("."));
    let source = read_file(&root.join("src/main.jalm"))?;
    let parsed = parse(&source);
    if !parsed.errors.is_empty() {
        return Err("parse errors in src/main.jalm".to_string());
    }
    let tc = check(&source);
    let ec = check_effects(&source);
    if !tc.diagnostics.is_empty() || !ec.diagnostics.is_empty() {
        return Err("check failed for src/main.jalm".to_string());
    }
    println!("run: ok (no runtime yet)");
    Ok(())
}

fn read_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))
}
