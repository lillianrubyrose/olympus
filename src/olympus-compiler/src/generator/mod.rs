pub mod rust;

use olympus_parser::{ParsedEnum, ParsedProcedure, ParsedRpcContainer, ParsedStruct};

pub trait CodeGenerator {
	fn generate_models(
		&self,
		parsed_enums: &[ParsedEnum],
		parsed_structs: &[ParsedStruct],
		parsed_rpc_containers: &[ParsedRpcContainer],
		output: &mut String,
	) {
		self.generate_file_header(output);

		for ele in parsed_enums {
			self.generate_enum(ele, output);
		}

		for ele in parsed_structs {
			self.generate_struct(ele, output);
		}

		for ele in parsed_rpc_containers {
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
