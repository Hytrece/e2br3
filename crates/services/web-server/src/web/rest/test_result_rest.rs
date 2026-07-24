use lib_core::model::test_result::{
	TestResultBmc, TestResultForCreate, TestResultForUpdate,
};
use lib_rest_core::prelude::*;
use lib_web::middleware::mw_auth::CtxW;

// Case-scoped CRUD functions:
// - create_test_result
// - get_test_result
// - list_test_results
// - update_test_result
// - delete_test_result
generate_case_rest_fns! {
	Bmc: TestResultBmc,
	Entity: lib_core::model::test_result::TestResult,
	ForCreate: TestResultForCreate,
	ForUpdate: TestResultForUpdate,
	Suffix: test_result
}

pub async fn restore_test_result(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<(
	StatusCode,
	Json<DataRestResult<lib_core::model::test_result::TestResult>>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("test_result:{id}"),
		move |ctx, mm| {
			Box::pin(async move {
				TestResultBmc::get_in_case_with_deleted(ctx, mm, case_id, id, true)
					.await?;
				TestResultBmc::restore_in_case(ctx, mm, case_id, id).await?;
				let data = TestResultBmc::get_in_case(ctx, mm, case_id, id).await?;
				Ok((StatusCode::OK, Json(DataRestResult { data })))
			})
		},
	)
	.await
}
