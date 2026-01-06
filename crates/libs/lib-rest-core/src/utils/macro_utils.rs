/// Create the base crud rpc functions following the common pattern.
/// - `create_...`
/// - `get_...`
///
/// NOTE: Make sure to import the Ctx, ModelManager, ... in the model that uses this macro.
/// 

#[macro_export]
macro_rules! generate_common_rest_fns {
    (
        Bmc: $bmc:ident,
        Entity: $entity:ty,
        ForCreate: $for_create:ty,
        ForUpdate: $for_update:ty,
        Filter: $filter:ty,
        Suffix: $suffix:ident
    ) => {
        paste! {
            pub async fn [<create_ $suffix>](
                ctx: Ctx,
                mm: ModelManager,
                Json(params): Json<ParamsForCreate<$for_create>>,
            ) -> Result<impl IntoResponse> {
                let ParamsForCreate { data } = params;
                let id = $bmc::create(&ctx, &mm, data).await?;
                let entity = $bmc::get(&ctx, &mm, id).await?;
                Ok($crate::rest_result::created(entity))
            }

            pub async fn [<get_ $suffix>](
                ctx: Ctx,
                mm: ModelManager,
                Path(id): Path<Uuid>,
            ) -> Result<impl IntoResponse> {
                let entity = $bmc::get(&ctx, &mm, id).await?;
                Ok($crate::rest_result::ok(entity))
            }

            // Note: for now just add `s` after the suffix.
            pub async fn [<list_ $suffix s>](
                ctx: Ctx,
                mm: ModelManager,
                Query(params): Query<ParamsList<$filter>>,
            ) -> Result<impl IntoResponse> {
                let entities = $bmc::list(&ctx, &mm, params.filters, params.list_options).await?;
                Ok($crate::rest_result::ok(entities))
            }

            pub async fn [<update_ $suffix>](
                ctx: Ctx,
                mm: ModelManager,
                Path(id): Path<Uuid>,
                Json(params): Json<ParamsForUpdate<$for_update>>,
            ) -> Result<impl IntoResponse> {
                let ParamsForUpdate { data } = params;
                $bmc::update(&ctx, &mm, id, data).await?;
                let entity = $bmc::get(&ctx, &mm, id).await?;
                Ok($crate::rest_result::ok(entity))
            }

            pub async fn [<delete_ $suffix>](
                ctx: Ctx,
                mm: ModelManager,
                Path(id): Path<Uuid>,
            ) -> Result<impl IntoResponse> {
                $bmc::delete(&ctx, &mm, id).await?;
                Ok($crate::rest_result::no_content())
            }
        }
    };
}
