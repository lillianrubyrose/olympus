use std::collections::HashMap;

use olympus_common::{ErrorColor, OlympusError, Spanned};
use olympus_parser::{
	ParsedEnum, ParsedEnumVariant, ParsedProcedure, ParsedProcedureParam, ParsedRpcContainer, ParsedStruct,
	ParsedStructField, Parser,
};

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

fn find_struct_field_duplicates(fields: &[ParsedStructField]) -> Result<(), OlympusError> {
	let mut idents = HashMap::<String, (Spanned<String>, Option<Spanned<String>>)>::new();

	for field in fields {
		if let Some((_, duplicated)) = idents.get_mut(&field.ident.value) {
			*duplicated = Some(field.ident.clone());
		} else {
			idents.insert(field.ident.value.clone(), (field.ident.clone(), None));
		}
	}

	for (_, (original_ident, dup_ident)) in idents {
		if let Some(dup_ident) = dup_ident {
			return Err(OlympusError::new("Duplicate field ident found")
				.label("Original here", original_ident.span, ErrorColor::Yellow)
				.label("Duplicate here", dup_ident.span, ErrorColor::Red));
		}
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
	let mut idents = HashMap::<String, (Spanned<String>, Option<Spanned<String>>)>::new();

	for param in params {
		if let Some((_, duplicated)) = idents.get_mut(&param.ident.value) {
			*duplicated = Some(param.ident.clone());
		} else {
			idents.insert(param.ident.value.clone(), (param.ident.clone(), None));
		}
	}

	for (_, (original_ident, dup_ident)) in idents {
		if let Some(dup_ident) = dup_ident {
			return Err(OlympusError::new("Duplicate proc param ident found")
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
	for ParsedEnum { variants, .. } in &parsed_enums {
		find_enum_variant_duplicates(variants)?;
	}

	for ParsedStruct { fields, .. } in &parsed_structs {
		find_struct_field_duplicates(fields)?;
	}

	for ParsedRpcContainer { procedures } in &parsed_rpc_containers {
		find_rpc_procedure_duplicates(procedures)?;
		for proc in procedures {
			find_rpc_procedure_param_duplicates(&proc.params)?;
		}
	}

	Ok(())
}
