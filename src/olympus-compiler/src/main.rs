use ariadne::{sources, Label, Report};
use olympus_common::OlympusError;
use olympus_lexer::Lexer;
use olympus_parser::Parser;
use olympus_verifier::verify_parser_outputs;

fn print_err<T>(src: &str, res: Result<T, OlympusError>) -> bool {
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
				Label::new(("example.olympus", label.span))
					.with_message(label.message)
					.with_color(label.color)
			})
			.collect::<Vec<_>>();

		Report::build(ariadne::ReportKind::Error, "example.olympus", lowest_start)
			.with_message(err.subject)
			.with_labels(labels)
			.finish()
			.print(sources([("example.olympus", src)]))
			.unwrap();
		return true;
	}
	false
}

fn main() {
	let src = include_str!("../assets/test.olympus");
	let mut lexer = Lexer::new(src);
	let lexer_err = print_err(src, lexer.lex());
	if lexer_err {
		println!("exited with lexer err");
		return;
	}

	let mut parser = Parser::new(lexer.tokens);
	let parser_err = print_err(src, parser.parse());
	if parser_err {
		println!("exited with parser err");
		return;
	}

	for ele in &parser.rpc_containers {
		for ele in &ele.procedures {
			dbg!(&ele.return_kind);
		}
	}
	// dbg!(&parser.rpc_containers);

	let verifier_err = print_err(src, verify_parser_outputs(parser));
	if verifier_err {
		println!("exited with verifier err");
	}
}
