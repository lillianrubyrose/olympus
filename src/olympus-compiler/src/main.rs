use ariadne::{sources, Color, Label, Report};
use olympus_common::SpannedErr;
use olympus_lexer::Lexer;
use olympus_parser::Parser;
use olympus_verifier::verify_parser_outputs;

fn print_err<T>(src: &str, res: Result<T, SpannedErr>) -> bool {
	if let Err(err) = res {
		Report::build(ariadne::ReportKind::Error, "example.olympus", err.span.start)
			.with_message(err.value.clone())
			.with_label(
				Label::new(("example.olympus", err.span))
					.with_message(err.value)
					.with_color(Color::Red),
			)
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

	let verifier_err = print_err(src, verify_parser_outputs(parser));
	if verifier_err {
		println!("exited with verifier err");
	}

	// Report::build(ariadne::ReportKind::Error, "example.olympus", 35)
	// 	.with_message("Duplicate variant value found in 'Action' enum")
	// 	.with_label(
	// 		Label::new(("example.olympus", 18..24))
	// 			.with_message("Original here")
	// 			.with_color(Color::Yellow),
	// 	)
	// 	.with_label(
	// 		Label::new(("example.olympus", 35..38))
	// 			.with_message("Duplicate here")
	// 			.with_color(Color::Red),
	// 	)
	// 	.finish()
	// 	.print(sources([("example.olympus", src)]))
	// 	.unwrap();
}
