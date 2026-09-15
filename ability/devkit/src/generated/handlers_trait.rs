// 自动生成，勿手改
use async_trait::async_trait;


#[derive(Debug, serde::Deserialize)]
pub struct ScaffoldArgs {}

#[derive(Debug, serde::Deserialize)]
pub struct ValidateArgs {}

#[derive(Debug, serde::Deserialize)]
pub struct GenArgs {}


#[async_trait]
pub trait Handlers: Send + Sync {

    async fn scaffold(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

    async fn validate(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

    async fn gen(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

}
