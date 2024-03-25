use olympus_lexer::IntToken;
use olympus_parser::{ParsedBultin, ParsedEnum, ParsedStruct, ParsedTypeKind};

use super::CodeGenerator;

pub struct RustCodeGenerator;

impl RustCodeGenerator {
	fn generate_enum_decl(parsed: &ParsedEnum, output: &mut String) {
		let variants = parsed
			.variants
			.iter()
			.map(|variant| format!("\t{} = {},", variant.ident.value, variant.value))
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum {} {{
{}
}}\n",
			parsed.ident.value, variants
		));
	}

	fn generate_enum_input_impl(parsed: &ParsedEnum, output: &mut String) {
		let match_branches = parsed
			.variants
			.iter()
			.map(|variant| format!("\t\t\t{} => Self::{},", variant.value, variant.ident.value))
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
impl ::olympus_net_common::ProcedureInput for {} {{
    fn deserialize(input: &mut ::olympus_net_common::bytes::BytesMut) -> Self {{
        use ::olympus_net_common::bytes::Buf;
        let tag = input.get_u16();
        match tag {{
{}
            tag => panic!(\"invalid tag: {{tag}}\"),
        }}
    }}
}}\n",
			parsed.ident.value, match_branches
		));
	}

	fn generate_enum_output_impl(parsed: &ParsedEnum, output: &mut String) {
		output.push_str(&format!(
			"
impl ::olympus_net_common::ProcedureOutput for {} {{
    fn serialize(&self) -> ::olympus_net_common::bytes::BytesMut {{
        use ::olympus_net_common::bytes::BufMut;
        let mut out = ::olympus_net_common::bytes::BytesMut::with_capacity(::std::mem::size_of::<u16>());
        out.put_u16(*self as _);
        out
    }}
}}\n",
			parsed.ident.value
		));
	}

	fn parsed_type_kind_to_rust(kind: &ParsedTypeKind) -> String {
		fn format_int(token: &IntToken) -> String {
			match token {
				IntToken::Int8 => "i8".to_string(),
				IntToken::Int16 => "i16".to_string(),
				IntToken::Int32 => "i32".to_string(),
				IntToken::Int64 => "i64".to_string(),
				IntToken::UInt8 => "u8".to_string(),
				IntToken::UInt16 => "u16".to_string(),
				IntToken::UInt32 => "u32".to_string(),
				IntToken::UInt64 => "u64".to_string(),
			}
		}

		match kind {
			ParsedTypeKind::Builtin(ty) => match ty {
				ParsedBultin::Int(int) => format_int(int),
				ParsedBultin::VariableInt(int) => format!("::olympus_net_common::Variable<{}>", format_int(int)),
				ParsedBultin::String => "String".to_string(),
				ParsedBultin::Array(ty) => format!("Vec<{}>", Self::parsed_type_kind_to_rust(&ty.value)),
				ParsedBultin::Option(ty) => format!("Option<{}>", Self::parsed_type_kind_to_rust(&ty.value)),
			},
			ParsedTypeKind::External(ident) => ident.to_string(),
		}
	}

	fn generate_struct_decl(parsed: &ParsedStruct, output: &mut String) {
		let fields = parsed
			.fields
			.iter()
			.map(|field| {
				format!(
					"\tpub {}: {},",
					field.ident.value,
					Self::parsed_type_kind_to_rust(&field.kind.value)
				)
			})
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
#[derive(Debug, Clone)]
pub struct {} {{
{}
}}\n",
			parsed.ident.value, fields
		));
	}

	fn generate_struct_input_impl(parsed: &ParsedStruct, output: &mut String) {
		let fields = parsed
			.fields
			.iter()
			.map(|field| {
				format!(
					"\t\t\t{}: ::olympus_net_common::ProcedureInput::deserialize(input),",
					field.ident.value
				)
			})
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
impl ::olympus_net_common::ProcedureInput for {} {{
    fn deserialize(input: &mut ::olympus_net_common::bytes::BytesMut) -> Self {{
        Self {{
{}
        }}
    }}
}}\n",
			parsed.ident.value, fields
		));
	}

	fn generate_struct_output_impl(parsed: &ParsedStruct, output: &mut String) {
		let fields = parsed
			.fields
			.iter()
			.map(|field| format!("\t\tout.extend(self.{}.serialize());", field.ident.value))
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
impl ::olympus_net_common::ProcedureOutput for {} {{
    fn serialize(&self) -> ::olympus_net_common::bytes::BytesMut {{
        let mut out = ::olympus_net_common::bytes::BytesMut::new();
{}
        out
    }}
}}\n",
			parsed.ident.value, fields
		));
	}
}

impl CodeGenerator for RustCodeGenerator {
	fn generate_enum(&self, parsed: &ParsedEnum, output: &mut String) {
		Self::generate_enum_decl(parsed, output);
		Self::generate_enum_input_impl(parsed, output);
		Self::generate_enum_output_impl(parsed, output);
	}

	fn generate_struct(&self, parsed: &ParsedStruct, output: &mut String) {
		Self::generate_struct_decl(parsed, output);
		Self::generate_struct_input_impl(parsed, output);
		Self::generate_struct_output_impl(parsed, output);
	}
}
