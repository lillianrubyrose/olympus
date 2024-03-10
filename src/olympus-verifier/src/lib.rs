use std::collections::HashMap;

use olympus_common::{ErrorColor, OlympusError, Spanned};
use olympus_parser::{ParsedEnum, ParsedEnumVariant, Parser};

fn find_enum_variant_duplicates(variants: &[ParsedEnumVariant]) -> Result<(), OlympusError> {
	let mut idents = HashMap::<String, (Spanned<String>, Option<Spanned<String>>)>::new();
	let mut values = HashMap::<i16, (Spanned<String>, Option<Spanned<String>>)>::new();

	for variant in variants {
		if let Some((_, duplicated)) = idents.get_mut(&variant.ident.value) {
			*duplicated = Some(variant.ident.clone());
		} else {
			idents.insert(variant.ident.value.clone(), (variant.ident.clone(), None));
		}

		if let Some((_, duplicated)) = values.get_mut(&variant.value) {
			*duplicated = Some(variant.ident.clone());
		} else {
			values.insert(variant.value, (variant.ident.clone(), None));
		}
	}

	for (_, (original_ident, dup_ident)) in idents {
		if let Some(dup_ident) = dup_ident {
			return Err(OlympusError::new("Duplicate variant ident found")
				.label("Original here", original_ident.span, ErrorColor::Yellow)
				.label("Duplicate here", dup_ident.span, ErrorColor::Red));
		}
	}

	for (_, (original_ident, dup_ident)) in values {
		if let Some(dup_ident) = dup_ident {
			return Err(OlympusError::new("Duplicate variant value found")
				.label("Original here", original_ident.span, ErrorColor::Yellow)
				.label("Duplicate here", dup_ident.span, ErrorColor::Red));
		}
	}

	Ok(())
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
