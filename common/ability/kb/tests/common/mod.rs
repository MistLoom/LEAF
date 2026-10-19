use std::path::PathBuf;

use ability_support::{InstanceId, LoadSpec, SourceId};
use frame_port_fun::FunPort;
use ability_core::AbilityHost;
use serde_json::json;

pub async fn build_kb_fun(root: impl Into<PathBuf>) -> FunPort {
    let root = root.into();
    let host = AbilityHost::new();
    host.install_str("store", ability_store::provider())
        .await
        .expect("install store");
    host.install_str("store.persist", ability_store_persist::provider())
        .await
        .expect("install persist");
    host.install_str("kb", ability_kb::provider())
        .await
        .expect("install kb");
    let store_spec = LoadSpec::new(
        SourceId::new("store").expect("src"),
        InstanceId::new("store").expect("inst"),
    )
    .with_config(json!({
        "data_root": root.join("data").to_string_lossy(),
        "temp_root": root.join("tmp").to_string_lossy(),
        "watch": false
    }));
    host.load(store_spec).await.expect("load store");
    host.load(LoadSpec::new(
        SourceId::new("store.persist").expect("src"),
        InstanceId::new("store.persist").expect("inst"),
    ))
    .await
    .expect("load persist");
    host.load(LoadSpec::new(
        SourceId::new("kb").expect("src"),
        InstanceId::new("kb").expect("inst"),
    ))
    .await
    .expect("load kb");
    FunPort::new(host.into_gateway())
}
