//! Agent Test 装配。

use frame_compose::{AppBuilder, AppHandle};

use crate::config::{
    default_paths, load_expose, load_provider, merge_chat_config, web_expose_path,
};

pub async fn bootstrap(
    expose_path: Option<&std::path::Path>,
    provider_path: Option<&std::path::Path>,
) -> Result<AppHandle, frame_core::Error> {
    let (def_expose, def_provider) = default_paths();
    let expose_path = expose_path.unwrap_or(&def_expose);
    let provider_path = provider_path.unwrap_or(&def_provider);

    let provider = load_provider(provider_path)?;
    let expose = merge_chat_config(load_expose(expose_path)?, provider);

    Ok(AppBuilder::new()
        .app("agent-test")
        .expose(expose)
        .bootstrap()
        .await)
}

/// WebUI 模式：默认加载 web expose（禁 `bash`）。
pub async fn bootstrap_web(
    expose_path: Option<&std::path::Path>,
    provider_path: Option<&std::path::Path>,
) -> Result<AppHandle, frame_core::Error> {
    let web_default = web_expose_path();
    let path = expose_path.unwrap_or(&web_default);
    bootstrap(Some(path), provider_path).await
}
