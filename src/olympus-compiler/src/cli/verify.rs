use std::{path::PathBuf, process::exit};

use crate::verify_src;

use super::{ensure_is_file, get_filename};

pub fn run(file: PathBuf) -> eyre::Result<()> {
	ensure_is_file(&file)?;

	let filename = get_filename(&file)?;
	let src = std::fs::read_to_string(file)?;
	if verify_src(&src, &filename).is_some() {
		println!("Valid!");
	} else {
		exit(-1);
	}

	Ok(())
}
