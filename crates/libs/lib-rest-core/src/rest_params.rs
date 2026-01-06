//! Base constructs for REST request parameters.
//!
//! Unlike RPC where all parameters come from a single JSON object, REST parameters
//! are extracted from different sources:
//! - Path parameters (e.g., `/api/agents/:id`)
//! - Query parameters (e.g., `/api/agents?filter=active`)
//! - Request body (JSON payload)
//!
//! These types are designed to work with Axum extractors.

use modql::filter::ListOptions;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};
use axum::extract::Path;
use uuid::Uuid;

/// Request body structure for REST Create calls.
/// Used with `Json<ParamsForCreate<D>>` extractor.
///
/// Example: POST /api/agents with body `{"data": {"name": "Agent Smith"}}`
#[derive(Debug, Deserialize)]
pub struct ParamsForCreate<D> {
	pub data: D,
}

/// Request body structure for REST Update calls.
/// Used with `Path<Uuid>` for ID and `Json<ParamsForUpdate<D>>` for data.
///
/// Example: PUT /api/agents/:id with body `{"data": {"name": "Updated Name"}}`
#[derive(Debug, Deserialize)]
pub struct ParamsForUpdate<D> {
	pub data: D,
}

/// Query parameters structure for REST List calls.
/// Used with `Query<ParamsList<F>>` extractor.
///
/// Example: GET /api/agents?filters=[{"field":"active","value":true}]&list_options={"limit":10}
#[serde_as]
#[derive(Debug, Deserialize, Default)]
pub struct ParamsList<F>
where
	F: DeserializeOwned,
{
	/// Filters can be a single filter or an array of filters
	#[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
	pub filters: Option<Vec<F>>,

	/// List options for pagination, sorting, etc.
	pub list_options: Option<ListOptions>,
}

pub type UuidPath = Path<Uuid>;