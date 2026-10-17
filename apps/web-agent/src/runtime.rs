//! Web 编排运行时。

use std::sync::Arc;

use ability_core::AbilityHost;
use frame_core::Gateway;

#[derive(Clone)]
pub struct Runtime {
    pub host: AbilityHost,
    pub gateway: Arc<dyn Gateway>,
    pub expose: frame_core::AbilityExpose,
}

impl From<frame_compose::AppHandle> for Runtime {
    fn from(app: frame_compose::AppHandle) -> Self {
        Self {
            host: app.host,
            gateway: app.gateway,
            expose: app.expose,
        }
    }
}

impl Runtime {
    pub fn ecosystem_enabled(&self) -> EcosystemFlags {
        EcosystemFlags {
            claude_mod: self
                .expose
                .abilities
                .get("claude_mod")
                .and_then(|e| e.enabled)
                .unwrap_or(false),
            dsh_plugin: self
                .expose
                .abilities
                .get("dsh_plugin")
                .and_then(|e| e.enabled)
                .unwrap_or(false),
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct EcosystemFlags {
    pub claude_mod: bool,
    pub dsh_plugin: bool,
}
