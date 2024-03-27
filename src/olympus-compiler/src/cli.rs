pub mod compile;
pub mod verify;

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use eyre::eyre;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
	#[command(subcommand)]
	pub command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompileLanguage {
	Rust,
}

#[derive(Subcommand)]
pub enum Command {
	/// Verify an olympus definition
	Verify { file: PathBuf },

	/// Compile an olympus definition
	Compile {
		/// Must point to an Olympus definition file
		input: PathBuf,
		/// The file or directory to write the output to. WARNING: WILL OVERWRITE!
		output: PathBuf,
		language: CompileLanguage,
		/// (Rust only) Generate a crate.
		#[arg(long)]
		rs_crate: bool,
		/// (Rust only) The name of the crate to generate.
		#[arg(long)]
		rs_crate_name: Option<String>,
	},
}

pub fn ensure_is_file(path: &Path) -> eyre::Result<()> {
	if !path.try_exists()? {
		return Err(eyre!("The provided path doesn't exist."));
	}

	if !path.is_file() {
		return Err(eyre!("The provided path doesn't lead to a file."));
	}

	Ok(())
}

pub fn get_filename(file: &Path) -> eyre::Result<String> {
	let Some(file_name) = file
		.file_name()
		.ok_or(eyre!("unreachable because file name cant end in '..'"))?
		.to_str()
	else {
		return Err(eyre!("File name contains invalid UTF-8"));
	};
	Ok(file_name.to_string())
}
