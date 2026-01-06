// region:    --- Modules

mod error;
pub mod rest_params;
pub mod rest_result;
mod utils;

pub use self::error::{Error, Result};
pub use rest_params::*;
pub use rest_result::*;

pub mod prelude;

// endregion: --- Modules
