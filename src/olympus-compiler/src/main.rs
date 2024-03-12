use ariadne::{sources, Label, Report};
use olympus_common::OlympusError;
use olympus_lexer::Lexer;
use olympus_parser::{ParsedTypeKind, Parser};
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

fn output_rust_type(ty: &ParsedTypeKind) -> String {
	match ty {
		olympus_parser::ParsedTypeKind::Builtin(ty) => match ty {
			olympus_parser::ParsedBultin::Int(int) => match int {
				olympus_lexer::IntToken::Int8 => "i8".to_string(),
				olympus_lexer::IntToken::Int16 => "i16".to_string(),
				olympus_lexer::IntToken::Int32 => "i32".to_string(),
				olympus_lexer::IntToken::Int64 => "i64".to_string(),
				olympus_lexer::IntToken::UInt8 => "u8".to_string(),
				olympus_lexer::IntToken::UInt16 => "u16".to_string(),
				olympus_lexer::IntToken::UInt32 => "u32".to_string(),
				olympus_lexer::IntToken::UInt64 => "u64".to_string(),
			},
			olympus_parser::ParsedBultin::VariableInt(_) => todo!("outputting varints isn't supported yet"),
			olympus_parser::ParsedBultin::String => "String".to_string(),
			olympus_parser::ParsedBultin::Array(ty) => format!("Vec<{}>", output_rust_type(&ty.value)),
		},
		olympus_parser::ParsedTypeKind::External(ident) => ident.to_string(),
	}
}

fn output_rust_models(
	Parser {
		enums,
		structs,
		rpc_containers: _,
		..
	}: &Parser,
) -> String {
	let mut res = String::new();

	for r#enum in enums {
		let mut e = String::new();
		e.push_str(&format!("#[repr(i16)]\nenum {} {{\n", &r#enum.ident.value));
		for variant in &r#enum.variants {
			e.push_str(&format!("\t{} = {},\n", &variant.ident.value, variant.value));
		}
		e.push_str("}\n\n");

		res.push_str(&e);
	}

	for strukt in structs {
		let mut e = String::new();
		e.push_str(&format!("struct {} {{\n", &strukt.ident.value));
		for field in &strukt.fields {
			e.push_str(&format!(
				"\tpub {}: {},\n",
				&field.ident.value,
				output_rust_type(&field.kind.value)
			));
		}
		e.push_str("}\n\n");

		res.push_str(&e);
	}

	res
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

	let verifier_err = print_err(src, verify_parser_outputs(&parser));
	if verifier_err {
		println!("exited with verifier err");
		return;
	}

	println!("{}", output_rust_models(&parser));
}
