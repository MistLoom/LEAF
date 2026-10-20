//! 组合根：Catalog、Bootstrap、Expose、装配与可选 Mgmt API。

pub mod app_control;
pub mod bootstrap;
pub mod host_lifecycle;
pub mod catalog;
pub mod cli_build;
pub mod compose;
pub mod config;
pub mod expose;
pub mod load_order;
pub mod protocol_registry;
mod registry;

pub use app_control::AppControl;
pub use bootstrap::{AppBuilder, AppHandle};
pub use host_lifecycle::{
    apply_registrations, filter_host_all, filter_host_invoke, filter_host_lifecycle,
    filter_host_load, filter_host_named, filter_host_unload, listen, on_host, on_host_message,
    HostRegistration,
};
pub use catalog::{
    ability_path, register_protocol_support, resolve_protocols, workspace_root, SourceEntry,
    WorkspaceCatalog,
};
pub use cli_build::build_clis;
pub use compose::{
    access_allowed, apply_expose_delta, bind_app_scope, build_gateway, build_gateway_with,
    install_providers, load_exposed, reload_one, ComposeArtifact, ComposeOptions, LoadOpts,
};
pub use config::ConfigCatalog;
pub use expose::ToolHost;
pub use load_order::{
    build_dep_graph, dep_from_value, expose_to_yaml, suggest_enable, topo_sort, unload_impact,
    write_expose_file, DepEdge, DepGraph, EnableSuggest, UnloadImpact,
};
#[cfg(feature = "port-fun")]
pub use compose::{build_fun_port, build_fun_port_with};
pub use protocol_registry::{
    any_supports_gate, build_tool_index, sync_protocol_support, McpToolEntry, ProtocolRegistry,
};

#[cfg(feature = "mgmt")]
pub mod mgmt;

/// 仓外接入：只依赖 `frame-compose`，勿直接依赖 `frame-core` 与各 port crate。
pub use frame_core::{AbilityExpose, AccessKind, ArtifactKind, LocalsScope};
#[cfg(feature = "port-fun")]
pub use frame_port_fun::FunPort;
