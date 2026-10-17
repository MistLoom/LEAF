//! 能力作者与引擎共用的契约。禁止依赖 frame-core / port。
//!
//! 跨能力调用的类型增强见 [`client`]。

mod ability;
mod aspect;
mod catalog;
mod context;
mod effect;
mod encoding;
mod error;
mod event;
pub mod feedback;
mod fn_ability;
mod handler;
mod id;
mod lifecycle;
mod load;
mod manifest;
mod spec;
mod status;
pub mod decl;
pub mod remote;
mod schema;
mod invoke_mask;

pub use ability::{Ability, AbilityProvider};
pub use ability_support_macros::{client, sdk_client};
pub use aspect::{Aspect, BoxInvoke, Next};
pub use catalog::{filter_method_catalog, instance_method_counts, method_catalog, resolve_target, CatalogEntry};
pub use context::AbilityCtx;
pub use effect::{Effect, Undo};
pub use encoding::{decode_text_bytes, decode_text_bytes_strict};
pub use error::{Error, ErrorKind, Result};
pub use event::{
    host_event, AbilityEvent, BusEvent, DiscoveryPayload, EventClass, EventFilter, EventHandler,
    EventListener, EventMiddleware, EventMode, EventNext, EventOpts, EventPipeline, EventReg,
    EventSink, EventUnreg, HostEvent, UserEvent, UserEventSink,
};
pub use feedback::{
    get, list, purge, register_processor, ArchLoc, FeedbackProcessor, FeedbackRecord,
    FeedbackScope, ProcessorReg, TagFilter, DETACHED_CHAIN,
};
pub use status::ChainStatus;
pub use fn_ability::{CtxFnAbility, FnAbility, StaticProvider};
pub use handler::{typed_method, typed_method_with_ctx};
pub use id::{
    AbilityId, AbilityIdError, InstanceId, InstanceIdError, LocalsScope, MethodId, MethodIdError,
    SourceId, SourceIdError,
};
pub use invoke_mask::InvokeMask;
pub use lifecycle::Lifecycle;
pub use load::LoadSpec;
pub use manifest::{AbilityManifest, Dependency, EventDecl, EventDir, MethodMeta};
pub use decl::{
    assert_clean_reports, detect_protocols, discover_ability_roots, lint_ability_package,
    lint_workspace, load_ability_decl, load_ability_package, load_cli_info, load_grpc_info,
    load_mcp_info, load_skill_info, load_tools_info, validate_framework_version, AbilityDecl,
    AbilityPackageMeta, CliArgDecl, CliInfo, FRAMEWORK_VERSION, GrpcInfo, GrpcRpcDecl, LintOptions,
    LintReport, MethodDecl, McpInfo, McpToolDecl, OutboundTopicDeclYaml, ProtocolKind, SkillInfo,
    ToolsInfo, ToolsMethodDecl,
};
pub use schema::{is_unrestricted_schema, validate_schema, SchemaError};
pub use spec::{box_fut, box_life_fut, AbilitySpec, FactoryProvider};

/// 带 ctx 的异步方法体。用于 [`AbilitySpec::on`]。
#[macro_export]
macro_rules! handler {
    (|$ctx:ident, $args:ident| $body:expr) => {
        move |$ctx: &dyn $crate::AbilityCtx, $args: ::serde_json::Value| {
            $crate::box_fut(async move { $body })
        }
    };
}

/// 投递反馈。位置由宏填入，链路取当前范围（无则 `detached`）。
///
/// 标签字面量：编译期校验（英文字母，长度 1..19）。  
/// 标签表达式：运行期丢弃非法项。
#[macro_export]
macro_rules! feedback {
    ([$($tag:literal),* $(,)?], $($t:tt)+) => {{
        $(
            const _: () = $crate::feedback::assert_tag($tag);
        )*
        $crate::feedback::submit(
            env!("CARGO_PKG_NAME"),
            module_path!(),
            &[$($tag),*],
            format!($($t)+),
        );
    }};
    ($tags:expr, $($t:tt)+) => {{
        let __tags = &$tags;
        let __tags: &[&str] = ::std::convert::AsRef::as_ref(__tags);
        $crate::feedback::submit(
            env!("CARGO_PKG_NAME"),
            module_path!(),
            __tags,
            format!($($t)+),
        );
    }};
}

/// 生命周期回调。用于 [`AbilitySpec::on_lifecycle`]。
#[macro_export]
macro_rules! life {
    (|$phase:ident, $ctx:ident| $body:expr) => {
        move |$phase: $crate::Lifecycle, $ctx: &dyn $crate::AbilityCtx| {
            $crate::box_life_fut(async move { $body })
        }
    };
}
