// 自动生成，勿手改
use async_trait::async_trait;


#[derive(Debug, serde::Deserialize)]
pub struct ScaffoldArgs {
    pub id: String,
    pub out: String,
#[serde(default)]
    pub protocols: Vec<String>,
    pub template: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ValidateArgs {
    pub path: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CodegenArgs {
    pub path: String,
}


#[async_trait]
pub trait Handlers: Send + Sync {

    async fn scaffold(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

    async fn validate(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

    async fn codegen(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

}
