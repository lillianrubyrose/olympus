use std::collections::HashMap;

use olympus_common::{ErrorColor, OlympusError, Spanned};
use olympus_parser::{
	ParsedEnum, ParsedEnumVariant, ParsedProcedure, ParsedProcedureParam, ParsedRpcContainer, ParsedStruct,
	ParsedStructField, Parser,
};

fn find_duplicate_ident(idents: &[Spanned<String>]) -> Option<(Spanned<String>, Spanned<String>)> {
	let mut idents_map = HashMap::<String, (Spanned<String>, Option<Spanned<String>>)>::new();

	for ident in idents {
		if let Some((_, duplicated)) = idents_map.get_mut(&ident.value) {
			*duplicated = Some(ident.clone());
		} else {
			idents_map.insert(ident.value.clone(), (ident.clone(), None));
		}
	}

	for (_, (original_ident, dup_ident)) in idents_map {
		if let Some(dup_ident) = dup_ident {
			return Some((original_ident, dup_ident));
		}
	}

	None
}

fn find_enum_variant_duplicates(variants: &[ParsedEnumVariant]) -> Result<(), OlympusError> {
	if let Some((original, dup)) = find_duplicate_ident(&variants.iter().map(|v| v.ident.clone()).collect::<Vec<_>>()) {
		return Err(OlympusError::new("Duplicate variant ident found")
			.label("Original here", original.span, ErrorColor::Yellow)
			.label("Duplicate here", dup.span, ErrorColor::Red));
	}

	let mut values = HashMap::<i16, (Spanned<String>, Option<Spanned<String>>)>::new();

	for variant in variants {
		if let Some((_, duplicated)) = values.get_mut(&variant.value) {
			*duplicated = Some(variant.ident.clone());
		} else {
			values.insert(variant.value, (variant.ident.clone(), None));
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

fn find_struct_field_duplicates(fields: &[ParsedStructField]) -> Result<(), OlympusError> {
	if let Some((original, dup)) = find_duplicate_ident(&fields.iter().map(|v| v.ident.clone()).collect::<Vec<_>>()) {
		return Err(OlympusError::new("Duplicate field ident found")
			.label("Original here", original.span, ErrorColor::Yellow)
			.label("Duplicate here", dup.span, ErrorColor::Red));
	}

	Ok(())
}

fn find_rpc_procedure_duplicates(procs: &[ParsedProcedure]) -> Result<(), OlympusError> {
	let mut idents = HashMap::<String, (Spanned<String>, Option<Spanned<String>>)>::new();

	for proc in procs {
		if let Some((_, duplicated)) = idents.get_mut(&proc.ident.value) {
			*duplicated = Some(proc.ident.clone());
		} else {
			idents.insert(proc.ident.value.clone(), (proc.ident.clone(), None));
		}
	}

	for (_, (original_ident, dup_ident)) in idents {
		if let Some(dup_ident) = dup_ident {
			return Err(OlympusError::new("Duplicate proc ident found")
				.label("Original here", original_ident.span, ErrorColor::Yellow)
				.label("Duplicate here", dup_ident.span, ErrorColor::Red));
		}
	}

	Ok(())
}

fn find_rpc_procedure_param_duplicates(params: &[ParsedProcedureParam]) -> Result<(), OlympusError> {
	if let Some((original, dup)) = find_duplicate_ident(&params.iter().map(|v| v.ident.clone()).collect::<Vec<_>>()) {
		return Err(OlympusError::new("Duplicate proc param ident found")
			.label("Original here", original.span, ErrorColor::Yellow)
			.label("Duplicate here", dup.span, ErrorColor::Red));
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
	for ParsedEnum { ident: _, variants } in &parsed_enums {
		find_enum_variant_duplicates(variants)?;
	}

	for ParsedStruct { ident: _, fields } in &parsed_structs {
		find_struct_field_duplicates(fields)?;
	}

	if let Some((original_ident, dup_ident)) = find_duplicate_ident(
		&parsed_enums
			.iter()
			.map(|v| v.ident.clone())
			.chain(parsed_structs.iter().map(|v| v.ident.clone()))
			.collect::<Vec<_>>(),
	) {
		return Err(OlympusError::new("Duplicate enum/struct ident found")
			.label("Original here", original_ident.span, ErrorColor::Yellow)
			.label("Duplicate here", dup_ident.span, ErrorColor::Red));
	}

	for ParsedRpcContainer { procedures } in &parsed_rpc_containers {
		find_rpc_procedure_duplicates(procedures)?;
		for proc in procedures {
			find_rpc_procedure_param_duplicates(&proc.params)?;
		}
	}

	Ok(())
}
