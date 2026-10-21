// 自动生成，勿手改
use async_trait::async_trait;


#[derive(Debug, serde::Deserialize)]
pub struct ChargeArgs {

}

#[derive(Debug, serde::Deserialize)]
pub struct PlaceArgs {

}

#[derive(Debug, serde::Deserialize)]
pub struct NoteArgs {

}

#[derive(Debug, serde::Deserialize)]
pub struct MixedArgs {

}


#[async_trait]
pub trait Handlers: Send + Sync {

    async fn charge(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

    async fn place(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

    async fn note(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

    async fn mixed(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

}
