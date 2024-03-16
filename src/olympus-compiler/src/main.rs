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
		let mut enum_declaration = String::new();
		let mut input_impl = String::new();
		let mut output_impl = String::new();

		enum_declaration.push_str(&format!("#[repr(i16)]\npub enum {} {{\n", &r#enum.ident.value));
		input_impl.push_str(&format!(
			"impl crate::callback::CallbackInput for {} {{\n",
			&r#enum.ident.value
		));
		input_impl.push_str("\tfn deserialize(input: &mut ::bytes::BytesMut) -> Self {\n");
		input_impl.push_str("\t\tuse ::bytes::Buf;\n");
		input_impl.push_str("\t\tlet tag = input.get_u16();\n");
		input_impl.push_str("\t\tmatch tag {\n");

		output_impl.push_str(&format!(
			"impl crate::callback::CallbackOutput for {} {{\n",
			&r#enum.ident.value
		));
		output_impl.push_str("\tfn serialize(self) -> ::bytes::BytesMut {\n");
		output_impl.push_str("\t\tuse ::bytes::BufMut;\n");
		output_impl.push_str("\t\tlet mut out = ::bytes::BytesMut::with_capacity(::std::mem::size_of::<u16>());\n");
		output_impl.push_str("\t\tout.put_u16(self as _);\n");
		output_impl.push_str("\t\tout\n");

		for variant in &r#enum.variants {
			enum_declaration.push_str(&format!("\t{} = {},\n", &variant.ident.value, variant.value));
			input_impl.push_str(&format!("\t\t\t{} => Self::{},\n", variant.value, variant.ident.value));
		}

		enum_declaration.push_str("}\n\n");
		input_impl.push_str("\t\t\t_ => panic!(\"invalid tag: {tag}\"),\n");
		input_impl.push_str("\t\t}\n\t}\n}\n\n");
		output_impl.push_str("\t}\n}\n\n");

		res.push_str(&enum_declaration);
		res.push_str(&input_impl);
		res.push_str(&output_impl);
	}

	for strukt in structs {
		let mut struct_declaration = String::new();
		let mut input_impl = String::new();
		let mut output_impl = String::new();

		struct_declaration.push_str(&format!("pub struct {} {{\n", &strukt.ident.value));

		input_impl.push_str(&format!(
			"impl crate::callback::CallbackInput for {} {{\n",
			&strukt.ident.value
		));
		input_impl.push_str("\tfn deserialize(input: &mut ::bytes::BytesMut) -> Self {\n");
		input_impl.push_str("\t\tSelf {\n");

		output_impl.push_str(&format!(
			"impl crate::callback::CallbackOutput for {} {{\n",
			&strukt.ident.value
		));
		output_impl.push_str("\tfn serialize(self) -> ::bytes::BytesMut {\n");
		output_impl.push_str("\t\tlet mut out = ::bytes::BytesMut::new();\n");

		for field in &strukt.fields {
			struct_declaration.push_str(&format!(
				"\tpub {}: {},\n",
				&field.ident.value,
				output_rust_type(&field.kind.value)
			));

			input_impl.push_str(&format!(
				"\t\t\t{}: crate::callback::CallbackInput::deserialize(input),\n",
				&field.ident.value
			));
			output_impl.push_str(&format!("\t\tout.extend(self.{}.serialize());\n", field.ident.value));
		}

		struct_declaration.push_str("}\n\n");
		input_impl.push_str("\t\t}\n\t}\n}\n\n");
		output_impl.push_str("\t\tout\n\t}\n}\n\n");

		res.push_str(&struct_declaration);
		res.push_str(&input_impl);
		res.push_str(&output_impl);
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
