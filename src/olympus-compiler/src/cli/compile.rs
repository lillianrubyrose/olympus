use std::{fs::OpenOptions, io::Write, path::PathBuf};

use crate::{
	generator::{rust::RustCodeGenerator, CodeGenerator},
	verify_src,
};
use eyre::eyre;
use olympus_parser::Parser;

use super::{ensure_is_file, get_filename, CompileLanguage};

pub fn run(
	input: PathBuf,
	output: PathBuf,
	language: CompileLanguage,
	rs_crate: bool,
	rs_crate_name: &Option<String>,
) -> eyre::Result<()> {
	ensure_is_file(&input)?;

	let filename = get_filename(&input)?;
	let src = std::fs::read_to_string(input)?;
	let Some(parser) = verify_src(&src, &filename) else {
		return Ok(());
	};

	match language {
		CompileLanguage::Rust => {
			gen_rust(&parser, output, rs_crate, rs_crate_name)?;
		}
	}

	println!("Compiled!");

	Ok(())
}

fn gen_rust(parser: &Parser, output: PathBuf, gen_crate: bool, gen_crate_name: &Option<String>) -> eyre::Result<()> {
	if gen_crate && gen_crate_name.is_none() {
		return Err(eyre!("Must specify crate name. (TIP: --rs-crate-name=<name>)"));
	} else if gen_crate {
		todo!()
	}

	if output.is_dir() {
		return Err(eyre!("You cannot output compiled source to a directory"));
	}

	let mut models_src = String::with_capacity(4096);
	RustCodeGenerator.generate_models(parser, &mut models_src);

	let mut output_file = OpenOptions::new()
		.write(true)
		.truncate(true)
		.create(true)
		.open(output)?;
	output_file.write_all(models_src.as_bytes())?;

	Ok(())
}
