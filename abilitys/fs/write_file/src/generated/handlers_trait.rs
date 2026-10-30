// 自动生成，勿手改
use async_trait::async_trait;


#[derive(Debug, serde::Deserialize)]
pub struct RunArgs {
#[serde(default)]
    pub bytes_base64: String,
#[serde(default)]
    pub contents: String,
#[serde(default)]
    pub overwrite: bool,
    pub path: String,
}


#[async_trait]
pub trait Handlers: Send + Sync {

    async fn run(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;

}
