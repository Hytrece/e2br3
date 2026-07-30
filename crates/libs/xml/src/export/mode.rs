use crate::Result;
use lib_core::ctx::Ctx;
use lib_core::model::case::Case;
use lib_core::model::ModelManager;
use lib_core::regulatory::RegulatoryAuthority;

pub(crate) use super::shared::postprocess::apply_section_postprocess;
use super::{base_export_skeleton, sections};

pub(crate) async fn try_fast_path_export(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	case: &Case,
	authority: RegulatoryAuthority,
) -> Result<Option<String>> {
	let Some(raw_xml) = case.raw_xml.as_deref() else {
		return Ok(None);
	};

	if is_only_dirty(case, "c") {
		return Ok(Some(
			sections::c::export_patch(ctx, mm, case_id, case, raw_xml, authority)
				.await?,
		));
	}
	if is_only_dirty(case, "d") {
		return Ok(Some(
			sections::d::export_patch(ctx, mm, case_id, raw_xml).await?,
		));
	}
	if is_only_dirty(case, "e") {
		return Ok(Some(
			sections::e::export_patch(mm, case_id, raw_xml, authority).await?,
		));
	}
	if is_only_dirty(case, "f") {
		return Ok(Some(sections::f::export_patch(mm, case_id, raw_xml).await?));
	}
	if is_only_dirty(case, "g") {
		return Ok(Some(
			sections::g::export_patch(mm, case_id, raw_xml, authority).await?,
		));
	}
	if is_only_dirty(case, "h") {
		return Ok(Some(
			sections::h::export_patch(ctx, mm, case_id, raw_xml).await?,
		));
	}

	Ok(None)
}

pub(crate) async fn build_fresh_export_from_db(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	case: &Case,
	authority: RegulatoryAuthority,
) -> Result<String> {
	let mut xml = base_export_skeleton().to_string();
	xml =
		sections::c::export_patch(ctx, mm, case_id, case, xml.as_bytes(), authority)
			.await?;
	xml = sections::d::export_patch(ctx, mm, case_id, xml.as_bytes()).await?;
	xml = sections::e::export_patch(mm, case_id, xml.as_bytes(), authority).await?;
	xml = sections::f::export_patch(mm, case_id, xml.as_bytes()).await?;
	xml = sections::g::export_patch(mm, case_id, xml.as_bytes(), authority).await?;
	sections::h::export_patch(ctx, mm, case_id, xml.as_bytes()).await
}

pub(crate) async fn apply_dirty_sections_from_db(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	case: &Case,
	mut xml: String,
	authority: RegulatoryAuthority,
) -> Result<String> {
	if case.dirty_c {
		xml = sections::c::export_patch(
			ctx,
			mm,
			case_id,
			case,
			xml.as_bytes(),
			authority,
		)
		.await?;
	}
	if case.dirty_d {
		xml = sections::d::export_patch(ctx, mm, case_id, xml.as_bytes()).await?;
	}
	if case.dirty_e {
		xml = sections::e::export_patch(mm, case_id, xml.as_bytes(), authority)
			.await?;
	}
	if case.dirty_f {
		xml = sections::f::export_patch(mm, case_id, xml.as_bytes()).await?;
	}
	if case.dirty_g {
		xml = sections::g::export_patch(mm, case_id, xml.as_bytes(), authority)
			.await?;
	}
	if case.dirty_h {
		xml = sections::h::export_patch(ctx, mm, case_id, xml.as_bytes()).await?;
	}
	Ok(xml)
}

fn is_only_dirty(case: &Case, section: &str) -> bool {
	match section {
		"c" => {
			case.dirty_c
				&& !case.dirty_d
				&& !case.dirty_e
				&& !case.dirty_f
				&& !case.dirty_g
				&& !case.dirty_h
		}
		"d" => {
			case.dirty_d
				&& !case.dirty_c
				&& !case.dirty_e
				&& !case.dirty_f
				&& !case.dirty_g
				&& !case.dirty_h
		}
		"e" => {
			case.dirty_e
				&& !case.dirty_c
				&& !case.dirty_d
				&& !case.dirty_f
				&& !case.dirty_g
				&& !case.dirty_h
		}
		"f" => {
			case.dirty_f
				&& !case.dirty_c
				&& !case.dirty_d
				&& !case.dirty_e
				&& !case.dirty_g
				&& !case.dirty_h
		}
		"g" => {
			case.dirty_g
				&& !case.dirty_c
				&& !case.dirty_d
				&& !case.dirty_e
				&& !case.dirty_f
				&& !case.dirty_h
		}
		"h" => {
			case.dirty_h
				&& !case.dirty_c
				&& !case.dirty_d
				&& !case.dirty_e
				&& !case.dirty_f
				&& !case.dirty_g
		}
		_ => false,
	}
}
