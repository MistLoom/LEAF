//! Ability `kb`：知识库多方法

mod core;
mod generated;
mod methods;
mod outbound;
mod store;

use std::sync::Arc;

use async_trait::async_trait;
use ability_support::{
    Ability, AbilityCtx, AbilityManifest, AbilityProvider, AbilitySpec, Lifecycle, MethodId,
    Result,
};
use serde_json::Value;

use crate::generated::manifest;
use crate::methods::KbMethods;
use crate::store::KbHub;

pub fn provider() -> Arc<dyn AbilityProvider> {
    AbilitySpec::from_manifest(manifest::manifest()).into_factory(|_config| async move {
        Ok(Arc::new(KbAbility::new()) as Arc<dyn Ability>)
    })
}

struct KbAbility {
    manifest: AbilityManifest,
    hub: Arc<KbHub>,
    methods: KbMethods,
}

impl KbAbility {
    fn new() -> Self {
        let hub = Arc::new(KbHub::new());
        Self {
            manifest: manifest::manifest(),
            methods: KbMethods::new(hub.clone()),
            hub,
        }
    }
}

#[async_trait]
impl Ability for KbAbility {
    fn manifest(&self) -> &AbilityManifest {
        &self.manifest
    }

    async fn on_lifecycle(&self, phase: Lifecycle, ctx: &dyn AbilityCtx) -> Result<()> {
        if phase == Lifecycle::Started {
            self.hub.bind_ctx(ctx.retain()).await;
        }
        Ok(())
    }

    async fn invoke(&self, method: &MethodId, _ctx: &dyn AbilityCtx, args: Value) -> Result<Value> {
        self.methods.invoke(method.as_str(), args).await
    }
}
