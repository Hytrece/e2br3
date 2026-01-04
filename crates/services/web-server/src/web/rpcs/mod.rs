// region:    --- Modules

use rpc_router::{Router, RouterBuilder};

// endregion: --- Modules

pub fn all_rpc_router_builder() -> RouterBuilder {
	// NOTE: Old agent_rpc and conv_rpc have been removed.
	// Add new RPC routers here for E2B(R3) SafetyDB models as needed.
	Router::builder()
}
