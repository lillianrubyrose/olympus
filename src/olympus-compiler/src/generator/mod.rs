use crate::cli::NamingConventionConfig;
use olympus_parser::{ParsedEnum, ParsedProcedure, ParsedRpcContainer, ParsedStruct, Parser};

pub mod rust;

pub trait CodeGenerator {
	fn generate_models(&self, parser: &Parser, output: &mut String, naming_convention_config: &NamingConventionConfig) {
		for r#enum in &parser.enums {
			self.generate_enum(r#enum, output, naming_convention_config);
		}

		for r#struct in &parser.structs {
			self.generate_struct(r#struct, output, naming_convention_config);
		}

		for proc in &parser.rpc_container.procedures {
			self.generate_procedure_params(proc, output, naming_convention_config);
		}
	}

	fn generate_file_header(&self, _output: &mut String) {}
	fn generate_file_footer(&self, _output: &mut String) {}
	fn generate_enum(
		&self,
		parsed: &ParsedEnum,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	);
	fn generate_struct(
		&self,
		parsed: &ParsedStruct,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	);
	fn generate_abstract_server_impl(
		&self,
		parsed: &ParsedRpcContainer,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	);
	fn generate_procedure_params(
		&self,
		parsed: &ParsedProcedure,
		output: &mut String,
		naming_convention_config: &NamingConventionConfig,
	);
}
