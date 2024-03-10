use ariadne::{sources, Color, Label, Report};
use olympus_lexer::{Lexer, SpannedErr};
use olympus_parser::Parser;

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

	dbg!(parser.rpc_containers);
}
