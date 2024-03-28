use crate::cli::NamingConventionConfig;
use olympus_lexer::IntToken;
use olympus_parser::{ParsedBultin, ParsedEnum, ParsedProcedure, ParsedRpcContainer, ParsedStruct, ParsedTypeKind};
use olympus_spanned::Spanned;

use super::CodeGenerator;

pub struct RustCodeGenerator;

impl RustCodeGenerator {
	fn generate_enum_decl(parsed: &ParsedEnum, output: &mut String, naming_convention_config: &NamingConventionConfig) {
		let variants = parsed
			.variants
			.iter()
			.map(|variant| {
				format!(
					"\t{} = {},",
					naming_convention_config.apply_enum_variants(&variant.ident.value),
					variant.value
				)
			})
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum {} {{
{variants}
}}\n",
			naming_convention_config.apply_types(&parsed.ident.value)
		));
	}

	fn generate_enum_input_impl(
		parsed: &ParsedEnum,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		let match_branches = parsed
			.variants
			.iter()
			.map(|variant| {
				format!(
					"\t\t\t{} => Ok(Self::{}),",
					variant.value,
					naming_convention_config.apply_enum_variants(&variant.ident.value)
				)
			})
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
impl ::olympus_net_common::ProcedureInput for {} {{
    fn deserialize(input: &mut ::olympus_net_common::bytes::BytesMut) -> ::olympus_net_common::Result<Self> {{
        use ::olympus_net_common::bytes::Buf;
        let tag = input.get_u16();
        match tag {{
{match_branches}
            _ => Err(::olympus_net_common::error!(\"invalid tag: {{tag}}\")),
        }}
    }}
}}\n",
			naming_convention_config.apply_types(&parsed.ident.value)
		));
	}

	fn generate_enum_output_impl(
		parsed: &ParsedEnum,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		output.push_str(&format!(
			"
impl ::olympus_net_common::ProcedureOutput for {} {{
    fn serialize(&self) -> ::olympus_net_common::Result<::olympus_net_common::bytes::BytesMut> {{
        use ::olympus_net_common::bytes::BufMut;
        let mut out = ::olympus_net_common::bytes::BytesMut::with_capacity(::std::mem::size_of::<u16>());
        out.put_u16(*self as _);
        Ok(out)
    }}
}}\n",
			naming_convention_config.apply_types(&parsed.ident.value)
		));
	}

	fn parsed_type_kind_to_rust(kind: &ParsedTypeKind, naming_convention_config: &NamingConventionConfig) -> String {
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
				ParsedBultin::Nothing => String::new(),
				ParsedBultin::Int(int) => format_int(int),
				ParsedBultin::VariableInt(int) => format!("::olympus_net_common::Variable<{}>", format_int(int)),
				ParsedBultin::String => "String".to_string(),
				ParsedBultin::Array(ty) => format!(
					"Vec<{}>",
					Self::parsed_type_kind_to_rust(&ty.value, naming_convention_config)
				),
				ParsedBultin::Option(ty) => format!(
					"Option<{}>",
					Self::parsed_type_kind_to_rust(&ty.value, naming_convention_config)
				),
			},
			ParsedTypeKind::External(ident) => naming_convention_config.apply_types(ident),
		}
	}

	fn generate_struct_decl(
		parsed: &ParsedStruct,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		let fields = parsed
			.fields
			.iter()
			.map(|field| {
				format!(
					"\tpub {}: {},",
					naming_convention_config.apply_struct_fields(&field.ident.value),
					Self::parsed_type_kind_to_rust(&field.kind.value, naming_convention_config)
				)
			})
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
#[derive(Debug, Clone)]
pub struct {} {{
{fields}
}}\n",
			naming_convention_config.apply_types(&parsed.ident.value)
		));
	}

	fn generate_struct_input_impl<F: Iterator<Item = Spanned<String>>>(
		ident: &str,
		fields: F,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		let fields = fields
			.map(|ident| {
				format!(
					"\t\t\t{}: ::olympus_net_common::ProcedureInput::deserialize(input)?,",
					naming_convention_config.apply_struct_fields(&ident.value)
				)
			})
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
impl ::olympus_net_common::ProcedureInput for {} {{
    fn deserialize(input: &mut ::olympus_net_common::bytes::BytesMut) -> ::olympus_net_common::Result<Self> {{
        Ok(Self {{
{fields}
        }})
    }}
}}\n",
			naming_convention_config.apply_types(ident)
		));
	}

	fn generate_struct_output_impl<F: Iterator<Item = Spanned<String>>>(
		ident: &str,
		fields: F,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		let fields = fields
			.map(|ident| {
				format!(
					"\t\tout.extend(self.{}.serialize()?);",
					naming_convention_config.apply_struct_fields(&ident.value)
				)
			})
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
impl ::olympus_net_common::ProcedureOutput for {} {{
    fn serialize(&self) -> ::olympus_net_common::Result<::olympus_net_common::bytes::BytesMut> {{
        let mut out = ::olympus_net_common::bytes::BytesMut::new();
{fields}
        Ok(out)
    }}
}}\n",
			naming_convention_config.apply_types(ident)
		));
	}
}

impl CodeGenerator for RustCodeGenerator {
	fn generate_file_header(&self, output: &mut String) {
		output.push_str("#![allow(unused_qualifications)]\n");
		output.push_str("#![allow(non_snake_case)]\n");
	}

	fn generate_enum(
		&self,
		parsed: &ParsedEnum,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		Self::generate_enum_decl(parsed, output, naming_convention_config);
		Self::generate_enum_input_impl(parsed, output, naming_convention_config);
		Self::generate_enum_output_impl(parsed, output, naming_convention_config);
	}

	fn generate_struct(
		&self,
		parsed: &ParsedStruct,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		let field_idents = parsed.fields.iter().map(|field| field.ident.clone());
		Self::generate_struct_decl(parsed, output, naming_convention_config);
		Self::generate_struct_input_impl(
			&parsed.ident.value,
			field_idents.clone(),
			output,
			naming_convention_config,
		);
		Self::generate_struct_output_impl(&parsed.ident.value, field_idents, output, naming_convention_config);
	}

	fn generate_abstract_server_impl(
		&self,
		parsed: &ParsedRpcContainer,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		let procedures = parsed
			.procedures
			.iter()
			.map(|proc| {
				let return_ty = Self::parsed_type_kind_to_rust(&proc.return_kind.value, naming_convention_config);
				let return_ty = if return_ty.is_empty() {
					"::olympus_net_common::Result<()>".to_string()
				} else {
					format!("::olympus_net_common::Result<{return_ty}>")
				};

				let proc_params = if proc.params.is_empty() {
					String::new()
				} else {
					format!(", params: {}Params", proc.ident.value)
				};

				format!(
					"\tasync fn {}(context: Ctx{proc_params}) -> {return_ty};",
					naming_convention_config.apply_procs(&proc.ident.value)
				)
			})
			.collect::<Vec<String>>()
			.join("\n");
		output.push_str("#[::olympus_net_common::async_trait]\n");
		output.push_str(&format!(
			"pub trait {}<Ctx: Clone + Send + Sync + 'static> {{\n",
			naming_convention_config.apply_types("ServerRpc")
		));
		output.push_str(&procedures);
		output.push_str("\n}\n");
	}

	fn generate_server_registration_fn(
		&self,
		parsed: &ParsedRpcContainer,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		output.push_str(&format!("pub async fn {}<C: Clone + Send + Sync + 'static, I: {}<C> + 'static>(server: &mut ::olympus_server::OlympusServer<C>, _imp: I) {{\n", naming_convention_config.apply_procs("register_procedures"), naming_convention_config.apply_types("ServerRpc")));
		for proc in &parsed.procedures {
			if proc.params.is_empty() {
				output.push_str(&format!(
					"\tserver.register_procedure(\"{}\", |ctx, (): ()| I::{}(ctx)).await;\n",
					proc.ident.value,
					naming_convention_config.apply_procs(&proc.ident.value)
				));
			} else {
				output.push_str(&format!(
					"\tserver.register_procedure(\"{}\", I::{}).await;\n",
					proc.ident.value,
					naming_convention_config.apply_procs(&proc.ident.value)
				));
			}
		}
		output.push_str("}\n");
	}

	fn generate_procedure_params(
		&self,
		parsed: &ParsedProcedure,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	) {
		if parsed.params.is_empty() {
			return;
		}

		let params = parsed
			.params
			.iter()
			.map(|param| {
				format!(
					"\tpub {}: {},",
					param.ident.value,
					Self::parsed_type_kind_to_rust(&param.kind.value, naming_convention_config)
				)
			})
			.collect::<Vec<String>>()
			.join("\n");

		output.push_str(&format!(
			"
#[derive(Debug, Clone)]
pub struct {}Params {{
{params}
}}
	",
			parsed.ident.value
		));

		let struct_ident = format!("{}Params", parsed.ident.value);
		let param_idents = parsed.params.iter().map(|field| field.ident.clone());
		Self::generate_struct_input_impl(&struct_ident, param_idents.clone(), output, naming_convention_config);
		Self::generate_struct_output_impl(&struct_ident, param_idents, output, naming_convention_config);
	}
}
