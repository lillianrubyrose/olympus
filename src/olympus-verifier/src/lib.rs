use std::collections::HashMap;

use olympus_common::{OlympusError, Spanned};
use olympus_parser::{ParsedEnum, ParsedEnumVariant, Parser};

fn find_enum_variant_duplicates(variants: &[ParsedEnumVariant]) -> Result<(), OlympusError> {
	variants
		.iter()
		.fold(HashMap::<i16, Option<Spanned<String>>, _>::new(), |mut acc, variant| {
			if let Some(duplicated) = acc.get_mut(&variant.value) {
				*duplicated = Some(variant.ident.clone());
			} else {
				acc.insert(variant.value, None);
			}
			acc
		})
		.into_iter()
		.try_fold((), |acc, (_, ident)| {
			if let Some(ident) = ident {
				return Err(OlympusError::error(
					"Enum with duplicated value found",
					ident.span.clone(),
				));
			}
			Ok(acc)
		})
}

pub fn verify_parser_outputs(
	Parser {
		enums: parsed_enums,
		structs: parsed_structs,
		rpc_containers: parsed_rpc_containers,
		..
	}: Parser,
) -> Result<(), OlympusError> {
	for ParsedEnum { ident, variants } in parsed_enums {
		find_enum_variant_duplicates(&variants)?;
	}

	Ok(())
}
