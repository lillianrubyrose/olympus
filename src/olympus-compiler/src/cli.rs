pub mod compile;
pub mod verify;

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use eyre::eyre;
use heck::{AsKebabCase, AsLowerCamelCase, AsPascalCase, AsShoutyKebabCase, AsShoutySnakeCase, AsSnakeCase};

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

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum NamingConvention {
	Pascal,
	LowerCamel,
	Snake,
	ShoutySnake,
	Kebab,
	ShoutyKebab,
}

#[derive(Subcommand)]
pub enum Command {
	/// Verify an olympus definition
	Verify { file: PathBuf },

	/// Compile an olympus definition
	Compile {
		/// Must point to an Olympus definition file
		input: PathBuf,
		/// The file or directory to write the output to.
		output: PathBuf,
		language: CompileLanguage,
		/// Overwrites files/directories if they're present instead of exiting.
		#[arg(short, long)]
		overwrite: bool,
		/// Overrides all other naming convention configuration to be this value.
		#[arg(long)]
		naming_convention: Option<NamingConvention>,
		/// What naming convention should be used for enums/structs
		#[arg(long, default_value = "pascal")]
		type_naming_convention: NamingConvention,
		/// What naming convention should be used for enum variants
		#[arg(long, default_value = "pascal")]
		enum_variant_naming_convention: NamingConvention,
		/// What naming convention should be used for struct fields
		#[arg(long, default_value = "snake")]
		struct_field_naming_convention: NamingConvention,
		/// What naming convention should be used for procedures
		#[arg(long, default_value = "snake")]
		proc_naming_convention: NamingConvention,
		/// (Rust only) Generate a crate.
		#[arg(long)]
		rs_crate: bool,
		/// (Rust only) The name of the crate to generate.
		#[arg(long)]
		rs_crate_name: Option<String>,
	},
}

#[derive(Debug, Clone)]
pub struct NamingConventionConfig {
	pub types: NamingConvention,
	pub struct_fields: NamingConvention,
	pub enum_variants: NamingConvention,
	pub procs: NamingConvention,
}

impl NamingConventionConfig {
	fn apply(conv: NamingConvention, input: &str) -> String {
		match conv {
			NamingConvention::Pascal => AsPascalCase(input).to_string(),
			NamingConvention::LowerCamel => AsLowerCamelCase(input).to_string(),
			NamingConvention::Snake => AsSnakeCase(input).to_string(),
			NamingConvention::ShoutySnake => AsShoutySnakeCase(input).to_string(),
			NamingConvention::Kebab => AsKebabCase(input).to_string(),
			NamingConvention::ShoutyKebab => AsShoutyKebabCase(input).to_string(),
		}
	}

	pub fn apply_types(&self, input: &str) -> String {
		Self::apply(self.types, input)
	}

	pub fn apply_enum_variants(&self, input: &str) -> String {
		Self::apply(self.enum_variants, input)
	}

	pub fn apply_struct_fields(&self, input: &str) -> String {
		Self::apply(self.struct_fields, input)
	}

	pub fn apply_procs(&self, input: &str) -> String {
		Self::apply(self.procs, input)
	}
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
