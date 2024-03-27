mod cli;
mod generator;

use ariadne::{sources, Label, Report};
use clap::Parser;
use olympus_common::OlympusError;
use std::process::exit;

fn print_olympus_error<T>(src: &str, filename: String, res: Result<T, OlympusError>) -> bool {
	if let Err(err) = res {
		let mut lowest_start = usize::MAX;
		for label in &err.labels {
			if label.span.start < lowest_start {
				lowest_start = label.span.start;
			}
		}

		let labels = err
			.labels
			.into_iter()
			.map(|label| {
				Label::new((filename.clone(), label.span))
					.with_message(label.message)
					.with_color(label.color)
			})
			.collect::<Vec<_>>();

		Report::build(ariadne::ReportKind::Error, filename.clone(), lowest_start)
			.with_message(err.subject)
			.with_labels(labels)
			.finish()
			.print(sources([(filename, src)]))
			.unwrap();
		return true;
	}
	false
}

#[must_use]
pub fn verify_src(src: &str, filename: &str) -> Option<olympus_parser::Parser> {
	let mut lexer = olympus_lexer::Lexer::new(src);
	if print_olympus_error(src, filename.to_string(), lexer.lex()) {
		return None;
	}

	let mut parser = olympus_parser::Parser::new(lexer.tokens);
	if print_olympus_error(src, filename.to_string(), parser.parse()) {
		return None;
	}

	if print_olympus_error(
		src,
		filename.to_string(),
		olympus_verifier::verify_parser_outputs(&parser),
	) {
		return None;
	}

	Some(parser)
}

fn main() {
	if let Err(err) = try_main() {
		eprintln!("{err}");
		exit(1);
	}
}

fn try_main() -> eyre::Result<()> {
	let args = cli::Args::parse();
	match args.command {
		cli::Command::Verify { file } => cli::verify::run(file)?,
		cli::Command::Compile {
			input,
			output,
			language,
			overwrite,
			rs_crate,
			rs_crate_name,
		} => {
			cli::compile::run(input, output, language, overwrite, rs_crate, &rs_crate_name)?;
		}
	}

	Ok(())
}
