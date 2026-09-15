//! 外部 Ability 示例。只依赖 `ability-sdk`，不依赖 frame。
//!
//! 覆盖：typed client / 别名调用 / `call!`、它方事件注入、自举 load、
//! 生命周期、切面、locals、可逆副作用、config、catalog / dep。

mod aspect;
mod echo;
mod hub;
mod note;
mod probe;
mod wrap;

pub use aspect::TraceAspect;
pub use echo::{echo_provider, EchoExt};
pub use hub::hub_provider;
pub use note::note_provider;
pub use probe::provider;
pub use wrap::{wrap_provider, WrapExt};
