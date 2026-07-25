use super::shared::*;

pub async fn create_product_presave(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Json(params): Json<ParamsForCreate<ProductPresaveForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<ProductPresave>>)> {
	let ctx = ctx_w.0;
	with_authorized_presave_create(
		&ctx,
		&snapshot,
		&mm,
		"product",
		move |ctx, mm| {
			Box::pin(async move {
				let ParamsForCreate { data } = params;
				let id = ProductPresaveBmc::create(ctx, mm, data).await?;
				Ok(rest_created(ProductPresaveBmc::get(ctx, mm, id).await?))
			})
		},
	)
	.await
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
	pub parent: ProductPresave,
	pub substances: Vec<ProductPresaveSubstance>,
}

#[derive(Deserialize)]
pub struct ProductPresaveDetailsForUpdate {
	pub parent: Option<ProductPresaveForUpdate>,
	pub substances: Option<Vec<ProductSubstanceDetailsForUpdate>>,
}

#[derive(Debug, Deserialize)]
pub struct ProductSubstanceDetailsForUpdate {
	pub id: Option<Uuid>,
	#[serde(default, rename = "_delete")]
	pub delete: bool,
	pub sequence_number: Option<i32>,
	pub substance_name: Option<String>,
	pub substance_termid_version: Option<String>,
	pub substance_termid: Option<String>,
	pub mfds_version: Option<String>,
	pub mfds_id: Option<String>,
	pub strength_value: Option<rust_decimal::Decimal>,
	pub strength_unit: Option<String>,
}

impl ProductSubstanceDetailsForUpdate {
	fn into_update(self) -> ProductPresaveSubstanceForUpdate {
		ProductPresaveSubstanceForUpdate {
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
	) -> Result<ProductPresaveSubstanceForCreate> {
		Ok(ProductPresaveSubstanceForCreate {
			product_presave_id,
			sequence_number: self.sequence_number.ok_or_else(|| {
				Error::BadRequest {
					message:
						"product substance details create requires sequence_number"
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
				if data
					.parent
					.as_ref()
					.is_some_and(|parent| parent.deleted == Some(true))
				{
					if data.substances.is_some() {
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
				preflight_product_presave_details(ctx, mm, id, &data).await?;
				apply_product_presave_details_inner(ctx, mm, id, data).await?;
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
	data: ProductPresaveDetailsForUpdate,
) -> Result<()> {
	if let Some(parent) = data.parent {
		ProductPresaveBmc::update(ctx, mm, id, parent).await?;
	}
	if let Some(substances) = data.substances {
		for substance in substances {
			upsert_product_substance_detail(ctx, mm, id, substance).await?;
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
	let substances = ProductPresaveSubstanceBmc::list_by_parent(ctx, mm, id).await?;
	Ok(ProductPresaveDetails { parent, substances })
}

async fn preflight_product_presave_details(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	product_id: Uuid,
	data: &ProductPresaveDetailsForUpdate,
) -> Result<()> {
	if let Some(substances) = &data.substances {
		for substance in substances {
			preflight_product_substance_detail(ctx, mm, product_id, substance)
				.await?;
		}
	}
	Ok(())
}

async fn preflight_product_substance_detail(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	product_id: Uuid,
	substance: &ProductSubstanceDetailsForUpdate,
) -> Result<()> {
	if substance.delete && substance.id.is_none() {
		return Err(Error::BadRequest {
			message: "product substance delete requires id".to_string(),
		});
	}
	if let Some(id) = substance.id {
		let entity = ProductPresaveSubstanceBmc::get(ctx, mm, id).await?;
		ensure_detail_parent_scope(
			product_id,
			entity.product_presave_id,
			id,
			"product",
			"product_presave_substances",
		)?;
	} else if !substance.delete {
		validate_product_substance_detail_create(substance)?;
	}
	Ok(())
}

fn validate_product_substance_detail_create(
	substance: &ProductSubstanceDetailsForUpdate,
) -> Result<()> {
	if substance.sequence_number.is_none() {
		return Err(Error::BadRequest {
			message: "product substance details create requires sequence_number"
				.to_string(),
		});
	}
	Ok(())
}

async fn upsert_product_substance_detail(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	product_id: Uuid,
	substance: ProductSubstanceDetailsForUpdate,
) -> Result<()> {
	if substance.delete && substance.id.is_none() {
		return Err(Error::BadRequest {
			message: "product substance delete requires id".to_string(),
		});
	}
	if let Some(id) = substance.id {
		let entity = ProductPresaveSubstanceBmc::get(ctx, mm, id).await?;
		ensure_detail_parent_scope(
			product_id,
			entity.product_presave_id,
			id,
			"product",
			"product_presave_substances",
		)?;
		if substance.delete {
			ProductPresaveSubstanceBmc::delete(ctx, mm, id).await?;
		} else {
			ProductPresaveSubstanceBmc::update(ctx, mm, id, substance.into_update())
				.await?;
		}
	} else {
		ProductPresaveSubstanceBmc::create(
			ctx,
			mm,
			substance.into_create(product_id)?,
		)
		.await?;
	}
	Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ProductSubstanceForRestCreate {
	pub sequence_number: i32,
	pub substance_name: Option<String>,
	pub substance_termid_version: Option<String>,
	pub substance_termid: Option<String>,
	pub mfds_version: Option<String>,
	pub mfds_id: Option<String>,
	pub strength_value: Option<rust_decimal::Decimal>,
	pub strength_unit: Option<String>,
}

impl ProductSubstanceForRestCreate {
	fn into_core(
		self,
		product_presave_id: Uuid,
	) -> ProductPresaveSubstanceForCreate {
		ProductPresaveSubstanceForCreate {
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
	Bmc: ProductPresaveSubstanceBmc,
	Entity: ProductPresaveSubstance,
	RestCreate: ProductSubstanceForRestCreate,
	ForUpdate: ProductPresaveSubstanceForUpdate,
	CreateFn: create_product_substance,
	ListFn: list_product_substances,
	GetFn: get_product_substance,
	UpdateFn: update_product_substance,
	DeleteFn: delete_product_substance,
	ParentField: product_presave_id,
	ParentKind: Product,
	EntityName: "product_presave_substances",
	DeleteMode: hard
}
