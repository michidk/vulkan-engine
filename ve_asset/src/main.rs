#![feature(once_cell)]
#![feature(str_split_once)]

pub(crate) mod mesh;
pub(crate) mod utils;

use anyhow::Result;
use log::{debug, error, warn};
use mesh::obj;
use std::path::Path;
use structopt::StructOpt;

// Cli arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "ve_asset")]
struct CliArgs {
    /// Specify the shader files to compile using glob
    glob: String,
    /// Output directory, to place the compiled shader in
    #[structopt(short = "o", long = "output")]
    output: String,
    /// Output debug info
    #[structopt(long = "verbose")]
    verbose: bool,
}

/// Happens during setup
#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error("Invalid glob pattern")]
    PatternError(#[from] glob::PatternError),
    #[error("Invalid glob")]
    GlobError(#[from] glob::GlobError),
    #[error("Output folder does not exist: {0}")]
    OutputFolderNonExistant(String),
}

/// Happens during shader compilation; prints the error and continues
#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error("Error reading the file")]
    FileRead(#[from] std::io::Error),
}

const GLOB_OPTIONS: glob::MatchOptions = glob::MatchOptions {
    case_sensitive: false,
    require_literal_separator: false,
    require_literal_leading_dot: false,
};

fn main() -> Result<()> {
    let args = CliArgs::from_args();

    if !args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    } else {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Debug)
            .init();
    }

    Ok(prepare(args)?)
}

fn prepare(args: CliArgs) -> Result<()> {
    let output_path = Path::new(&args.output);
    // check if output folder exists
    if !output_path.exists() && !output_path.is_dir() {
        return Err(CliError::OutputFolderNonExistant(
            output_path
                .to_str()
                .expect("Invalid output path")
                .to_owned(),
        )
        .into());
    }

    let glob = glob::glob_with(&args.glob, GLOB_OPTIONS)?;
    for path in glob {
        let path = path?;

        if path.is_dir() {
            continue;
        }

        // check extension
        if let Some(Some(extension)) = path.extension().map(|x| x.to_str()) {
            match extension.to_ascii_lowercase().as_ref() {
                "obj" => obj::process(&path, &output_path)?,
                "toml" => debug!("Ignored toml file: {}", &path.display()),
                _ => warn!("Could not handle path: {}", &path.display()),
            }
        } else {
            warn!(
                "Ignored file \"{}\", because no file extension was found.",
                path.display()
            );
        }
    }

    Ok(())
}
