//! Ability 作者 SDK。独立外部工程，只依赖 `ability-support`，禁止依赖 `frame-core` / port。
//!
//! 跨能力调用：对方 crate 用 [`client`] 声明方法签名；本侧 `use FooExt` 后
//! `ctx.foo().method(typed_args)`（声明合并）。动态 id 仍可用 `call!` / [`AbilityRef`]。

mod invoke;
mod guest;
mod cli;

pub use ability_support::{
    sdk_client as client, host_event, Ability, AbilityCtx, AbilityId, AbilityManifest,
    AbilityProvider, AbilitySpec, Aspect, BusEvent, CatalogEntry, ChainStatus, CtxFnAbility,
    Dependency, DiscoveryPayload, Effect, Error, ErrorKind, EventClass, EventDecl, EventDir,
    EventFilter, EventHandler, EventListener, EventMiddleware, EventMode, EventNext, EventOpts,
    EventPipeline, EventReg, EventSink, FactoryProvider, FeedbackProcessor, FeedbackRecord, FeedbackScope,
    FnAbility, InstanceId, Lifecycle, LoadSpec, MethodId, MethodMeta, Next, Result, SourceId,
    StaticProvider, TagFilter, Undo, box_fut, box_life_fut,
};
pub use invoke::{AbilityRef, MethodRef};
pub use guest::{serve, serve_stdio};
pub use cli::build_standalone;

pub mod prelude {
    pub use super::invoke::AbilityRef;
    pub use super::{
        client, AbilityCtx, AbilityProvider, AbilitySpec, Aspect, Error, Lifecycle, Result,
        ability, call, emit, feedback, handler, life, method_id,
    };
    pub use serde_json::{json, Value};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __id {
    ($last:ident) => {
        stringify!($last)
    };
    ($head:ident . $($tail:ident).+) => {
        concat!(stringify!($head), ".", $crate::__id!($($tail).+))
    };
}

#[macro_export]
macro_rules! method_id {
    ($($part:ident).+) => {
        $crate::MethodId::new($crate::__id!($($part).+)).expect("方法标识")
    };
}

#[macro_export]
macro_rules! call {
    ($ctx:ident . $($ability:ident).+ :: $($method:ident).+ ($args:expr)) => {
        $ctx.call(
            $crate::__id!($($ability).+),
            &$crate::method_id!($($method).+),
            $args,
        )
    };
}

#[macro_export]
macro_rules! ability {
    ($ctx:ident . $($id:ident).+) => {
        $crate::AbilityRef::new($ctx, $crate::__id!($($id).+))
    };
}

#[macro_export]
macro_rules! emit {
    ($ctx:ident . $($name:ident).+ ($payload:expr)) => {
        $ctx.emit($crate::__id!($($name).+), $payload)
    };
}

#[macro_export]
macro_rules! feedback {
    ($($t:tt)*) => {
        ::ability_support::feedback!($($t)*);
    };
}

#[macro_export]
macro_rules! handler {
    (|$ctx:ident, $args:ident| $body:expr) => {
        move |$ctx: &dyn $crate::AbilityCtx, $args: ::serde_json::Value| {
            $crate::box_fut(async move { $body })
        }
    };
}

#[macro_export]
macro_rules! life {
    (|$phase:ident, $ctx:ident| $body:expr) => {
        move |$phase: $crate::Lifecycle, $ctx: &dyn $crate::AbilityCtx| {
            $crate::box_life_fut(async move { $body })
        }
    };
}
