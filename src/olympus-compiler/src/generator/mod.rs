use olympus_parser::{ParsedEnum, ParsedProcedure, ParsedStruct, Parser};

pub mod rust;

pub trait CodeGenerator {
	fn generate_models(&self, parser: &Parser, output: &mut String) {
		self.generate_file_header(output);

		for ele in &parser.enums {
			self.generate_enum(ele, output);
		}

		for ele in &parser.structs {
			self.generate_struct(ele, output);
		}

		for ele in &parser.rpc_containers {
			for ele in &ele.procedures {
				self.generate_procedure_params(ele, output);
			}
		}

		self.generate_file_footer(output);
	}

	fn generate_file_header(&self, _output: &mut String) {}
	fn generate_file_footer(&self, _output: &mut String) {}
	fn generate_enum(&self, parsed: &ParsedEnum, output: &mut String);
	fn generate_struct(&self, parsed: &ParsedStruct, output: &mut String);
	fn generate_procedure_params(&self, parsed: &ParsedProcedure, output: &mut String);
}
