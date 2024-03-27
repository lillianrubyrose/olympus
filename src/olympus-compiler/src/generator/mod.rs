use olympus_parser::{ParsedEnum, ParsedProcedure, ParsedRpcContainer, ParsedStruct, Parser};

pub mod rust;

pub trait CodeGenerator {
	fn generate_models(&self, parser: &Parser, output: &mut String) {
		self.generate_file_header(output);

		self.generate_abstract_server_impl(&parser.rpc_container, output);

		for r#enum in &parser.enums {
			self.generate_enum(r#enum, output);
		}

		for r#struct in &parser.structs {
			self.generate_struct(r#struct, output);
		}

		for proc in &parser.rpc_container.procedures {
			self.generate_procedure_params(proc, output);
		}

		self.generate_file_footer(output);
	}

	fn generate_file_header(&self, _output: &mut String) {}
	fn generate_file_footer(&self, _output: &mut String) {}
	fn generate_enum(&self, parsed: &ParsedEnum, output: &mut String);
	fn generate_struct(&self, parsed: &ParsedStruct, output: &mut String);
	fn generate_abstract_server_impl(&self, parsed: &ParsedRpcContainer, output: &mut String);
	fn generate_procedure_params(&self, parsed: &ParsedProcedure, output: &mut String);
}
