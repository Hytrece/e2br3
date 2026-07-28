use super::rows::camelize_value;
use super::shared::*;
use serde_json::{json, Value};

pub async fn create_product_presave(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Json(params): Json<ParamsForCreate<ProductPresaveRowsForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<ProductPresaveDetails>>)> {
	let ctx = ctx_w.0;
	with_authorized_presave_atomic_create(
		&ctx,
		&snapshot,
		&mm,
		"product",
		move |ctx, mm| {
			Box::pin(async move {
				let ParamsForCreate { data } = params;
				for substance in &data.rows.active_substances {
					validate_product_active_substance_detail_create(substance)?;
					if substance.deleted {
						return Err(Error::BadRequest {
							message:
								"new product active substance cannot be deleted"
									.into(),
						});
					}
				}
				let id =
					ProductPresaveBmc::create(ctx, mm, data.rows.product).await?;
				for substance in data.rows.active_substances {
					ProductPresaveActiveSubstanceBmc::create(
						ctx,
						mm,
						substance.into_create(id)?,
					)
					.await?;
				}
				Ok(rest_created(
					load_product_presave_details(ctx, mm, id).await?,
				))
			})
		},
	)
	.await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductPresaveRowsForCreate {
	pub rows: ProductPresaveCreateRows,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductPresaveCreateRows {
	pub product: ProductPresaveForCreate,
	#[serde(default)]
	pub active_substances: Vec<ProductActiveSubstanceDetailsForUpdate>,
}

pub async fn list_product_presaves(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
) -> Result<(StatusCode, Json<DataRestResult<Vec<ProductPresave>>>)> {
	let ctx = ctx_w.0;
	with_authorized_presave_collection(&ctx, &snapshot, &mm, |ctx, mm, scope| {
		Box::pin(async move {
			let entities = ProductPresaveBmc::list(ctx, mm, None).await?;
			Ok(rest_ok(filter_product_presaves_for_scope(scope, entities)))
		})
	})
	.await
}

pub async fn get_product_presave(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<ProductPresave>>)> {
	let ctx = ctx_w.0;
	with_authorized_presave_read(
		&ctx,
		&snapshot,
		&mm,
		PresaveAuthorizationKind::Product,
		id,
		|ctx, mm| {
			Box::pin(async move {
				Ok(rest_ok(ProductPresaveBmc::get(ctx, mm, id).await?))
			})
		},
	)
	.await
}

pub async fn update_product_presave(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(id): Path<Uuid>,
	Json(params): Json<ParamsForUpdate<ProductPresaveForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<ProductPresave>>)> {
	let ctx = ctx_w.0;
	with_authorized_presave_update(
		&ctx,
		&snapshot,
		&mm,
		PresaveAuthorizationKind::Product,
		id,
		move |ctx, mm| {
			Box::pin(async move {
				let ParamsForUpdate { data } = params;
				if data.deleted == Some(true) {
					PresaveLifecycleService::archive(
						ctx,
						mm,
						PresaveKind::Product,
						id,
					)
					.await?;
				} else {
					ProductPresaveBmc::update(ctx, mm, id, data).await?;
				}
				Ok(rest_ok(ProductPresaveBmc::get(ctx, mm, id).await?))
			})
		},
	)
	.await
}

pub async fn delete_product_presave(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(id): Path<Uuid>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	with_authorized_presave_update(
		&ctx,
		&snapshot,
		&mm,
		PresaveAuthorizationKind::Product,
		id,
		|ctx, mm| {
			Box::pin(async move {
				PresaveLifecycleService::archive(ctx, mm, PresaveKind::Product, id)
					.await?;
				Ok(StatusCode::NO_CONTENT)
			})
		},
	)
	.await
}

#[derive(Debug, Serialize)]
pub struct ProductPresaveDetails {
	pub id: Uuid,
	pub rows: ProductPresaveRows,
	// Transitional response keys; removed after Product callers migrate.
	pub parent: Value,
	pub active_substances: Vec<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPresaveRows {
	pub product: Value,
	pub active_substances: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductPresaveDetailsForUpdate {
	pub rows: Option<ProductPresaveRowsForUpdate>,
	// Transitional request keys; removed after Product callers migrate.
	pub parent: Option<ProductPresaveForUpdate>,
	#[serde(alias = "active_substances")]
	pub active_substances: Option<Vec<ProductActiveSubstanceDetailsForUpdate>>,
}

impl ProductPresaveDetailsForUpdate {
	fn into_rows(self) -> Result<ProductPresaveRowsForUpdate> {
		if let Some(rows) = self.rows {
			if self.parent.is_some() || self.active_substances.is_some() {
				return Err(Error::BadRequest {
					message: "product rows cannot be mixed with legacy detail keys"
						.into(),
				});
			}
			return Ok(rows);
		}
		Ok(ProductPresaveRowsForUpdate {
			product: self.parent,
			active_substances: self.active_substances,
		})
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductPresaveRowsForUpdate {
	pub product: Option<ProductPresaveForUpdate>,
	pub active_substances: Option<Vec<ProductActiveSubstanceDetailsForUpdate>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductActiveSubstanceDetailsForUpdate {
	pub id: Option<Uuid>,
	#[serde(default, alias = "_delete")]
	pub deleted: bool,
	#[serde(alias = "sequence_number")]
	pub sequence_number: Option<i32>,
	#[serde(alias = "substance_name")]
	pub substance_name: Option<String>,
	#[serde(alias = "substance_termid_version")]
	pub substance_termid_version: Option<String>,
	#[serde(alias = "substance_termid")]
	pub substance_termid: Option<String>,
	#[serde(alias = "mfds_version")]
	pub mfds_version: Option<String>,
	#[serde(alias = "mfds_id")]
	pub mfds_id: Option<String>,
	#[serde(rename = "substanceStrengthValue", alias = "strength_value")]
	pub strength_value: Option<rust_decimal::Decimal>,
	#[serde(rename = "substanceStrengthUnit", alias = "strength_unit")]
	pub strength_unit: Option<String>,
}

impl ProductActiveSubstanceDetailsForUpdate {
	fn into_update(self) -> ProductPresaveActiveSubstanceForUpdate {
		ProductPresaveActiveSubstanceForUpdate {
			sequence_number: self.sequence_number,
			substance_name: self.substance_name,
			substance_termid_version: self.substance_termid_version,
			substance_termid: self.substance_termid,
			mfds_version: self.mfds_version,
			mfds_id: self.mfds_id,
			strength_value: self.strength_value,
			strength_unit: self.strength_unit,
		}
	}

	fn into_create(
		self,
		product_presave_id: Uuid,
	) -> Result<ProductPresaveActiveSubstanceForCreate> {
		Ok(ProductPresaveActiveSubstanceForCreate {
			product_presave_id,
			sequence_number: self.sequence_number.ok_or_else(|| {
				Error::BadRequest {
					message:
						"product active substance details create requires sequence_number"
							.to_string(),
				}
			})?,
			substance_name: self.substance_name,
			substance_termid_version: self.substance_termid_version,
			substance_termid: self.substance_termid,
			mfds_version: self.mfds_version,
			mfds_id: self.mfds_id,
			strength_value: self.strength_value,
			strength_unit: self.strength_unit,
		})
	}
}

pub async fn get_product_presave_details(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<ProductPresaveDetails>>)> {
	let ctx = ctx_w.0;
	with_authorized_presave_read(
		&ctx,
		&snapshot,
		&mm,
		PresaveAuthorizationKind::Product,
		id,
		|ctx, mm| {
			Box::pin(async move {
				Ok(rest_ok(load_product_presave_details(ctx, mm, id).await?))
			})
		},
	)
	.await
}

pub async fn update_product_presave_details(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(id): Path<Uuid>,
	Json(params): Json<ParamsForUpdate<ProductPresaveDetailsForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<ProductPresaveDetails>>)> {
	let ctx = ctx_w.0;
	with_authorized_presave_atomic_update(
		&ctx,
		&snapshot,
		&mm,
		PresaveAuthorizationKind::Product,
		id,
		move |ctx, mm| {
			Box::pin(async move {
				let ParamsForUpdate { data } = params;
				let rows = data.into_rows()?;
				if rows
					.product
					.as_ref()
					.is_some_and(|parent| parent.deleted == Some(true))
				{
					if rows.active_substances.is_some() {
						return Err(Error::BadRequest {
							message: "presave deletion cannot include child changes"
								.into(),
						});
					}
					PresaveLifecycleService::archive(
						ctx,
						mm,
						PresaveKind::Product,
						id,
					)
					.await?;
					return Ok(rest_ok(
						load_product_presave_details(ctx, mm, id).await?,
					));
				}
				preflight_product_presave_details(ctx, mm, id, &rows).await?;
				apply_product_presave_details_inner(ctx, mm, id, rows).await?;
				Ok(rest_ok(load_product_presave_details(ctx, mm, id).await?))
			})
		},
	)
	.await
}

async fn apply_product_presave_details_inner(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	id: Uuid,
	rows: ProductPresaveRowsForUpdate,
) -> Result<()> {
	if let Some(parent) = rows.product {
		ProductPresaveBmc::update(ctx, mm, id, parent).await?;
	}
	if let Some(active_substances) = rows.active_substances {
		for substance in active_substances {
			upsert_product_active_substance_detail(ctx, mm, id, substance).await?;
		}
	}
	Ok(())
}

async fn load_product_presave_details(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	id: Uuid,
) -> Result<ProductPresaveDetails> {
	let parent = ProductPresaveBmc::get(ctx, mm, id).await?;
	let active_substances =
		ProductPresaveActiveSubstanceBmc::list_by_parent(ctx, mm, id).await?;
	let parent_value =
		serde_json::to_value(parent).map_err(|err| Error::BadRequest {
			message: format!("product presave serialization failed: {err}"),
		})?;
	let product = camelize_value(parent_value.clone());
	let legacy_active_substances = active_substances
		.iter()
		.map(|row| {
			serde_json::to_value(row).expect("serializable product substance")
		})
		.collect();
	let canonical_active_substances = active_substances
		.into_iter()
		.map(|substance| {
			let mut value = camelize_value(
				serde_json::to_value(substance)
					.expect("serializable product substance"),
			);
			if let Some(row) = value.as_object_mut() {
				if let Some(strength) = row.remove("strengthValue") {
					row.insert("substanceStrengthValue".into(), strength);
				}
				if let Some(unit) = row.remove("strengthUnit") {
					row.insert("substanceStrengthUnit".into(), unit);
				}
				row.insert("deleted".into(), json!(false));
			}
			value
		})
		.collect();
	Ok(ProductPresaveDetails {
		id,
		rows: ProductPresaveRows {
			product,
			active_substances: canonical_active_substances,
		},
		parent: parent_value,
		active_substances: legacy_active_substances,
	})
}

async fn preflight_product_presave_details(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	product_id: Uuid,
	rows: &ProductPresaveRowsForUpdate,
) -> Result<()> {
	if let Some(active_substances) = &rows.active_substances {
		for substance in active_substances {
			preflight_product_active_substance_detail(
				ctx, mm, product_id, substance,
			)
			.await?;
		}
	}
	Ok(())
}

async fn preflight_product_active_substance_detail(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	product_id: Uuid,
	substance: &ProductActiveSubstanceDetailsForUpdate,
) -> Result<()> {
	if substance.deleted && substance.id.is_none() {
		return Err(Error::BadRequest {
			message: "product active substance delete requires id".to_string(),
		});
	}
	if let Some(id) = substance.id {
		let entity = ProductPresaveActiveSubstanceBmc::get(ctx, mm, id).await?;
		ensure_detail_parent_scope(
			product_id,
			entity.product_presave_id,
			id,
			"product",
			"product_presave_active_substances",
		)?;
	} else if !substance.deleted {
		validate_product_active_substance_detail_create(substance)?;
	}
	Ok(())
}

fn validate_product_active_substance_detail_create(
	substance: &ProductActiveSubstanceDetailsForUpdate,
) -> Result<()> {
	if substance.sequence_number.is_none() {
		return Err(Error::BadRequest {
			message:
				"product active substance details create requires sequence_number"
					.to_string(),
		});
	}
	Ok(())
}

async fn upsert_product_active_substance_detail(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	product_id: Uuid,
	substance: ProductActiveSubstanceDetailsForUpdate,
) -> Result<()> {
	if substance.deleted && substance.id.is_none() {
		return Err(Error::BadRequest {
			message: "product active substance delete requires id".to_string(),
		});
	}
	if let Some(id) = substance.id {
		let entity = ProductPresaveActiveSubstanceBmc::get(ctx, mm, id).await?;
		ensure_detail_parent_scope(
			product_id,
			entity.product_presave_id,
			id,
			"product",
			"product_presave_active_substances",
		)?;
		if substance.deleted {
			ProductPresaveActiveSubstanceBmc::delete(ctx, mm, id).await?;
		} else {
			ProductPresaveActiveSubstanceBmc::update(
				ctx,
				mm,
				id,
				substance.into_update(),
			)
			.await?;
		}
	} else {
		ProductPresaveActiveSubstanceBmc::create(
			ctx,
			mm,
			substance.into_create(product_id)?,
		)
		.await?;
	}
	Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductActiveSubstanceForRestCreate {
	pub sequence_number: i32,
	pub substance_name: Option<String>,
	pub substance_termid_version: Option<String>,
	pub substance_termid: Option<String>,
	pub mfds_version: Option<String>,
	pub mfds_id: Option<String>,
	pub strength_value: Option<rust_decimal::Decimal>,
	pub strength_unit: Option<String>,
}

impl ProductActiveSubstanceForRestCreate {
	fn into_core(
		self,
		product_presave_id: Uuid,
	) -> ProductPresaveActiveSubstanceForCreate {
		ProductPresaveActiveSubstanceForCreate {
			product_presave_id,
			sequence_number: self.sequence_number,
			substance_name: self.substance_name,
			substance_termid_version: self.substance_termid_version,
			substance_termid: self.substance_termid,
			mfds_version: self.mfds_version,
			mfds_id: self.mfds_id,
			strength_value: self.strength_value,
			strength_unit: self.strength_unit,
		}
	}
}

generate_presave_child_rest_fns! {
	Bmc: ProductPresaveActiveSubstanceBmc,
	Entity: ProductPresaveActiveSubstance,
	RestCreate: ProductActiveSubstanceForRestCreate,
	ForUpdate: ProductPresaveActiveSubstanceForUpdate,
	CreateFn: create_product_active_substance,
	ListFn: list_product_active_substances,
	GetFn: get_product_active_substance,
	UpdateFn: update_product_active_substance,
	DeleteFn: delete_product_active_substance,
	ParentField: product_presave_id,
	ParentKind: Product,
	EntityName: "product_presave_active_substances",
	DeleteMode: hard
}
