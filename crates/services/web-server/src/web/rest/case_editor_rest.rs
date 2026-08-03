//! Case editor REST handlers, split by editor surface.
//!
//! Shared DTO/import helpers live in `common`; handler modules are re-exported
//! so existing router paths (`case_editor_rest::<handler>`) stay unchanged.

mod common;

mod ae;
mod dg;
mod dh;
mod direct;
mod input_contract_save;
mod lb;
mod lr;
mod shell;

pub use ae::*;
pub use dg::*;
pub use dh::*;
pub use direct::*;
pub(crate) use input_contract_save::validate_row_payload;
pub use lb::*;
pub use lr::*;
pub use shell::*;
