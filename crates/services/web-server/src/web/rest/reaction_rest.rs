use lib_core::model::reaction::{ReactionBmc, ReactionForCreate, ReactionForUpdate};
use lib_rest_core::prelude::*;

// Case-scoped CRUD functions:
// - create_reaction
// - get_reaction
// - list_reactions
// - update_reaction
// - delete_reaction
generate_case_rest_fns! {
	Bmc: ReactionBmc,
	Entity: lib_core::model::reaction::Reaction,
	ForCreate: ReactionForCreate,
	ForUpdate: ReactionForUpdate,
	Suffix: reaction
}

pub async fn restore_reaction(
	State(mm): State<ModelManager>,
	ctx_w: lib_web::middleware::mw_auth::CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<(
	StatusCode,
	Json<DataRestResult<lib_core::model::reaction::Reaction>>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("reaction:{id}"),
		move |ctx, mm| {
			Box::pin(async move {
				ReactionBmc::get_in_case_with_deleted(ctx, mm, case_id, id, true)
					.await?;
				ReactionBmc::restore_in_case(ctx, mm, case_id, id).await?;
				let entity = ReactionBmc::get_in_case(ctx, mm, case_id, id).await?;
				Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
			})
		},
	)
	.await
}
