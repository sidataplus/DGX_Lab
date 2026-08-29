#![forbid(unsafe_code)]

//! Build-time compiler for human-authored DGX Lab scenario YAML.

use clap::Parser;
use scenarios::ScenarioDefinition;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "dgxlab-scenario-compiler")]
#[command(about = "Validate and compile DGX Lab scenario YAML into deterministic JSON packs")]
struct Args {
    /// Source YAML file or directory.
    #[arg(long)]
    input: PathBuf,
    /// Output directory.
    #[arg(long)]
    output: PathBuf,
    /// Validate only; do not write compiled files.
    #[arg(long, default_value_t = false)]
    check: bool,
}

#[derive(Debug, Serialize)]
struct CompiledScenario<'a> {
    schema: &'static str,
    compiler_version: &'static str,
    source_digest_sha256: String,
    scenario: &'a ScenarioDefinition,
}

fn main() -> Result<(), CompilerError> {
    let args = Args::parse();
    let sources = discover_yaml(&args.input)?;
    if sources.is_empty() {
        return Err(CompilerError::NoSources(args.input));
    }
    if !args.check {
        fs::create_dir_all(&args.output)?;
    }
    for source in sources {
        let bytes = fs::read(&source)?;
        let scenario: ScenarioDefinition = serde_yaml::from_slice(&bytes)
            .map_err(|source_error| CompilerError::Yaml { path: source.clone(), source: source_error })?;
        validate(&scenario, &source)?;
        let digest = hex::encode(Sha256::digest(&bytes));
        let compiled = CompiledScenario {
            schema: "dgxlab.compiled-scenario/v1",
            compiler_version: env!("CARGO_PKG_VERSION"),
            source_digest_sha256: digest,
            scenario: &scenario,
        };
        if !args.check {
            let path = args.output.join(format!("{}.json", scenario.id));
            fs::write(path, serde_json::to_vec_pretty(&compiled)?)?;
        }
        println!("validated {} ({})", scenario.id, source.display());
    }
    Ok(())
}

fn discover_yaml(input: &Path) -> Result<Vec<PathBuf>, CompilerError> {
    if input.is_file() {
        return Ok(vec![input.to_path_buf()]);
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(input)? {
        let path = entry?.path();
        if path.is_file()
            && path.extension().and_then(|value| value.to_str()).is_some_and(|ext| {
                ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml")
            })
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn validate(scenario: &ScenarioDefinition, path: &Path) -> Result<(), CompilerError> {
    if scenario.schema != "dgxlab.scenario/v1" {
        return Err(CompilerError::Validation {
            path: path.to_path_buf(),
            message: format!("unsupported scenario schema: {}", scenario.schema),
        });
    }
    if scenario.id.trim().is_empty() || scenario.revision.trim().is_empty() {
        return Err(CompilerError::Validation {
            path: path.to_path_buf(),
            message: "id and revision are required".into(),
        });
    }
    if scenario.learner.username.contains('/') || scenario.learner.username.contains('\\') {
        return Err(CompilerError::Validation {
            path: path.to_path_buf(),
            message: "learner username may not contain path separators".into(),
        });
    }
    for file in &scenario.initial_files {
        if !file.path.starts_with('/') || file.path.split('/').any(|part| part == "..") {
            return Err(CompilerError::Validation {
                path: path.to_path_buf(),
                message: format!("unsafe virtual path: {}", file.path),
            });
        }
    }
    let mut objective_ids = std::collections::BTreeSet::new();
    for objective in &scenario.objectives {
        if !objective_ids.insert(&objective.id) {
            return Err(CompilerError::Validation {
                path: path.to_path_buf(),
                message: format!("duplicate objective id: {}", objective.id),
            });
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum CompilerError {
    #[error("no scenario YAML files found under {0}")]
    NoSources(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid YAML in {path}: {source}")]
    Yaml { path: PathBuf, source: serde_yaml::Error },
    #[error("scenario validation failed for {path}: {message}")]
    Validation { path: PathBuf, message: String },
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}
