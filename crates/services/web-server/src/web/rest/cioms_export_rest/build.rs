use super::*;

pub(super) fn ordered_cioms_case_data(
	data: &CiomsCaseData,
	settings: &CiomsSettings,
) -> CiomsCaseData {
	let mut ordered = data.clone();
	if settings
		.data_ordering
		.eq_ignore_ascii_case("Latest data will appear first")
	{
		ordered.reactions.reverse();
		ordered.drugs.reverse();
		ordered.dosages.reverse();
		ordered.indications.reverse();
		ordered.test_results.reverse();
		ordered.primary_sources.reverse();
		ordered.senders.reverse();
	}
	ordered
}

pub(super) fn build_cioms_pdf(
	data: &CiomsCaseData,
	settings: &CiomsSettings,
) -> Vec<u8> {
	build_cioms_pdf_with_options(data, settings, CiomsExportOptions::default())
}

pub(super) fn build_cioms_pdf_with_options(
	data: &CiomsCaseData,
	settings: &CiomsSettings,
	options: CiomsExportOptions,
) -> Vec<u8> {
	let (width, height) = if settings.orientation == "Portrait" {
		(595, 842)
	} else {
		(
			CIOMS_LANDSCAPE_TEMPLATE.page_width,
			CIOMS_LANDSCAPE_TEMPLATE.page_height,
		)
	};
	let ordered = ordered_cioms_case_data(data, settings);
	let mut canvas = PdfCanvas::new();
	canvas.stream.push_str("0.8 w\n");
	if settings.orientation == "Portrait" {
		let template = CIOMS_LANDSCAPE_TEMPLATE;
		let scale = (width as f32 / template.page_width as f32)
			.min(height as f32 / template.page_height as f32);
		let translate_x = (width as f32 - template.page_width as f32 * scale) / 2.0;
		let translate_y =
			(height as f32 - template.page_height as f32 * scale) / 2.0;
		canvas.save_state();
		canvas.transform(scale, scale, translate_x, translate_y);
		render_landscape_cioms(
			&mut canvas,
			&ordered,
			settings,
			options,
			width,
			height,
		);
		canvas.restore_state();
	} else {
		render_landscape_cioms(
			&mut canvas,
			&ordered,
			settings,
			options,
			width,
			height,
		);
	}
	let first_stream = canvas.stream;
	let overflow = collect_cioms_overflow(&ordered, settings, options);
	let continuation_streams = render_cioms_continuation_pages(
		&ordered.case_number,
		&overflow,
		width,
		height,
	);

	let obj1 = "<< /Type /Catalog /Pages 2 0 R >>";
	let page_refs = std::iter::once(3)
		.chain((0..continuation_streams.len()).map(|index| 8 + (index * 2)))
		.map(|page| format!("{page} 0 R"))
		.collect::<Vec<_>>()
		.join(" ");
	let page_count = 1 + continuation_streams.len();
	let obj2 = format!("<< /Type /Pages /Kids [{page_refs}] /Count {page_count} >>");
	let obj3 = format!(
		"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /Contents 7 0 R >>"
	);
	let obj4 = "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
	let obj5 = "<< /Type /Font /Subtype /Type0 /BaseFont /HYSMyeongJo-Medium /Encoding /UniKS-UCS2-H /DescendantFonts [6 0 R] >>";
	let obj6 = "<< /Type /Font /Subtype /CIDFontType0 /BaseFont /HYSMyeongJo-Medium /CIDSystemInfo << /Registry (Adobe) /Ordering (Korea1) /Supplement 2 >> /DW 1000 >>";
	let obj7 = format!(
		"<< /Length {} >>\nstream\n{}endstream",
		first_stream.len(),
		first_stream
	);
	let mut objects = vec![
		obj1.to_string(),
		obj2,
		obj3,
		obj4.to_string(),
		obj5.to_string(),
		obj6.to_string(),
		obj7,
	];
	let mut page_object = 8;
	for stream in continuation_streams {
		objects.push(format!(
			"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /Contents {} 0 R >>",
			page_object + 1
		));
		objects.push(format!(
			"<< /Length {} >>\nstream\n{}endstream",
			stream.len(),
			stream
		));
		page_object += 2;
	}

	let mut pdf = String::from("%PDF-1.4\n");
	let mut offsets = Vec::with_capacity(objects.len());
	for (idx, object) in objects.iter().enumerate() {
		offsets.push(pdf.len());
		pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", idx + 1, object));
	}
	let xref_offset = pdf.len();
	pdf.push_str("xref\n");
	pdf.push_str(&format!("0 {}\n", objects.len() + 1));
	pdf.push_str("0000000000 65535 f \n");
	for offset in offsets {
		pdf.push_str(&format!("{offset:010} 00000 n \n"));
	}
	pdf.push_str(&format!(
		"trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
		objects.len() + 1
	));
	pdf.into_bytes()
}

pub async fn export_case_cioms_pdf(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(id): Path<Uuid>,
	Query(query): Query<ExportCiomsQuery>,
) -> Result<Response> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_export(
		&ctx,
		&snapshot,
		&mm,
		&[id],
		move |ctx, mm| {
			Box::pin(async move {
				let settings = load_cioms_settings(ctx, mm).await?;
				let data = load_cioms_case_data(ctx, mm, id).await?;
				let pdf = build_cioms_pdf_with_options(
					&data,
					&settings,
					CiomsExportOptions {
						include_notation: query
							.include_notation
							.unwrap_or(settings.notation),
					},
				);
				let file_name = format!("{}-cioms.pdf", data.case_number);

				let mut response = (StatusCode::OK, pdf).into_response();
				response.headers_mut().insert(
					header::CONTENT_TYPE,
					header::HeaderValue::from_static("application/pdf"),
				);
				response.headers_mut().insert(
					header::CONTENT_DISPOSITION,
					header::HeaderValue::from_str(&format!(
						"attachment; filename=\"{file_name}\""
					))
					.map_err(|err| Error::BadRequest {
						message: format!("invalid CIOMS filename header: {err}"),
					})?,
				);
				Ok(response)
			})
		},
	)
	.await
}
