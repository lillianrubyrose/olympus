use std::{fs::OpenOptions, io::Write, path::PathBuf};

use crate::{
	generator::{rust::RustCodeGenerator, CodeGenerator},
	verify_src,
};
use eyre::eyre;
use olympus_parser::Parser;

use super::{ensure_is_file, get_filename, CompileLanguage};

pub fn run(file: PathBuf, output: PathBuf, language: CompileLanguage) -> eyre::Result<()> {
	ensure_is_file(&file)?;

	if output.is_dir() {
		return Err(eyre!("You cannot output compiled source to a directory"));
	}

	let filename = get_filename(&file)?;
	let src = std::fs::read_to_string(file)?;
	let Some(Parser {
		enums: parsed_enums,
		structs: parsed_structs,
		rpc_containers: parsed_rpc_containers,
		..
	}) = verify_src(&src, &filename)
	else {
		return Ok(());
	};

	let source_gen = match language {
		CompileLanguage::Rust => RustCodeGenerator,
	};

	let mut models_src = String::with_capacity(4096);
	source_gen.generate_models(&parsed_enums, &parsed_structs, &parsed_rpc_containers, &mut models_src);

	let mut output_file = OpenOptions::new()
		.write(true)
		.truncate(true)
		.create(true)
		.open(output)?;
	output_file.write_all(models_src.as_bytes())?;

	println!("Compiled!");

	Ok(())
}
