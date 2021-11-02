#![feature(once_cell)]

pub(crate) mod mesh;
pub(crate) mod utils;

use anyhow::Result;
use log::{debug, error, warn};
use mesh::obj;
use std::{fs, io, path::Path};
use structopt::StructOpt;
use walkdir::WalkDir;

// Cli arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "ve_asset")]
struct CliArgs {
    /// Specify the input folder
    input: String,
    /// Output directory, to place the transpiled files in
    #[structopt(short = "o", long = "output")]
    output: String,
    /// Output debug info
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}

/// Happens during setup
#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error("Input folder does not exist: {0}")]
    InputFolderNonExistant(String),
    #[error("Output folder strcuture could not be created: {0}")]
    ErrorCreatingOutputStructure(#[from] io::Error),
}

/// Happens during shader compilation; prints the error and continues
#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error("Error reading the file")]
    FileRead(#[from] std::io::Error),
}

fn main() -> Result<()> {
    let args = CliArgs::from_args();

    if !args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    } else {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Debug)
            .init();
    }

    prepare(args)
}

fn prepare(args: CliArgs) -> Result<()> {
    let output_path = Path::new(&args.output);

    let input_path = Path::new(&args.input);
    if !input_path.exists() && !input_path.is_dir() {
        return Err(CliError::InputFolderNonExistant(
            input_path.to_str().expect("Invalid input path").to_owned(),
        )
        .into());
    }

    for entry in WalkDir::new(input_path) {
        let path = match &entry {
            Err(err) => {
                warn!("Error parsing path: {}", err);
                continue;
            }
            Ok(entry) => entry.path(),
        };

        if path.is_dir() {
            continue;
        }

        let output = output_path.join(
            path.strip_prefix(input_path)
                .expect("Error handling output path: stripping prefix"),
        );

        // creating the output folder of the input file in the same structure
        let local_output_folder = output.parent().unwrap_or(output_path);
        if !local_output_folder.exists() {
            fs::create_dir_all(&local_output_folder)
                .map_err(CliError::ErrorCreatingOutputStructure)?;
        }

        // check extension
        if let Some(Some(extension)) = path.extension().map(|x| x.to_str()) {
            match extension.to_ascii_lowercase().as_ref() {
                "obj" => obj::process(path, local_output_folder)?,
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
