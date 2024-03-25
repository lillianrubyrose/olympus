pub mod rust;

use olympus_parser::{ParsedEnum, ParsedStruct};

pub trait CodeGenerator {
	fn generate_models(&self, parsed_enums: &[ParsedEnum], parsed_structs: &[ParsedStruct], output: &mut String) {
		for ele in parsed_enums {
			self.generate_enum(ele, output);
		}

		for ele in parsed_structs {
			self.generate_struct(ele, output);
		}
	}

	fn generate_enum(&self, parsed: &ParsedEnum, output: &mut String);
	fn generate_struct(&self, parsed: &ParsedStruct, output: &mut String);
}
