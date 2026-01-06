//! This is a prelude for REST handler modules to avoid redundant imports.

pub use crate::generate_common_rest_fns;
pub use crate::rest_result::{DataRestResult, created, ok, no_content};
pub use crate::rest_params::{ParamsForCreate, ParamsForUpdate, ParamsList};
pub use crate::Result;
pub use lib_core::ctx::Ctx;
pub use lib_core::model::ModelManager;
pub use paste::paste;
pub use axum::{
    extract::{Path, Json, Query},
    response::IntoResponse,
};
pub use uuid::Uuid;
