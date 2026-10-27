//! 双进程参考示例（无业务语义，假能力）：
//!
//! - 只读 Guest 子进程（Node，`guest/`）：`notebook.read` / `notebook.ping`
//! - 宿主侧写子能力（进程内假能力）：`scratch.append`
//! - 宿主经统一 Gateway 调用两侧；演示推荐的跨进程拼法
//!
//! 运行：`cargo run -p example-dual-process`（需要 Node.js）

use std::path::PathBuf;
use std::sync::Arc;

use ability_core::{AbilityHost, GuestSourceProvider};
use ability_support::{
    Ability, AbilityManifest, AbilityProvider, FnAbility, InstanceId, LoadSpec, MethodId,
    SourceId, StaticProvider,
};
use async_trait::async_trait;
use frame_core::{Gateway, InvokeRequest, Port};
use frame_port_http::HttpPort;
use serde_json::{Value, json};

/// 宿主侧写子假能力：内存追加，返回当前条数。
struct ScratchProvider;

#[async_trait]
impl AbilityProvider for ScratchProvider {
    fn manifest(&self) -> AbilityManifest {
        AbilityManifest {
            id: ability_support::AbilityId::new("scratch").unwrap(),
            depends: vec![],
            stateful: false,
            config_schema: Value::Null,
            methods: vec![ability_support::MethodMeta {
                id: MethodId::new("append").unwrap(),
                description: "写子假能力：追加一条，返回条数".into(),
                input_schema: Value::Null,
                output_schema: None,
            }],
            events: vec![],
        }
    }

    async fn create(&self, _config: Value) -> ability_support::Result<Arc<dyn Ability>> {
        let count = Arc::new(tokio::sync::Mutex::new(0u64));
        Ok(Arc::new(FnAbility::new(self.manifest(), move |_args: Value| {
            let count = Arc::clone(&count);
            async move {
                let mut n = count.lock().await;
                *n += 1;
                Ok(json!({ "count": *n }))
            }
        })))
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = AbilityHost::new();

    // 1) 只读 Guest 子进程（Node）——经 stdio NDJSON 线协议接入
    let guest_provider =
        GuestSourceProvider::from_workspace_rel(&workspace_root(), "examples/dual-process/guest")
            .into_provider();
    host.install(SourceId::new("notebook").unwrap(), guest_provider)
        .await;
    host.load(LoadSpec::new(
        SourceId::new("notebook").unwrap(),
        InstanceId::new("notebook").unwrap(),
    ))
    .await?;

    // 2) 宿主侧写子假能力（进程内）
    host.install(SourceId::new("scratch").unwrap(), Arc::new(StaticProvider::new(
        Arc::new(ScratchProvider).create(json!({})).await?,
    )))
    .await;
    host.load(LoadSpec::new(
        SourceId::new("scratch").unwrap(),
        InstanceId::new("scratch").unwrap(),
    ))
    .await?;

    // 3) 经统一 Gateway 调用两侧（AbilityHost 即 Gateway 实现）
    let read = host
        .invoke(InvokeRequest::new(
            InstanceId::new("notebook").unwrap(),
            MethodId::new("read").unwrap(),
            json!({}),
        ))
        .await?;
    println!("guest read → {}", read.data);

    let write = host
        .invoke(InvokeRequest::new(
            InstanceId::new("scratch").unwrap(),
            MethodId::new("append").unwrap(),
            json!({}),
        ))
        .await?;
    println!("host append → {}", write.data);

    // 4) 可选：把两侧同时投影为 HTTP 面（含 /health 与统一 invoke）
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    drop(listener);
    println!("可选 HTTP 面：HttpPort::new({addr}) 后 serve 即可投影两侧能力");
    let _ = HttpPort::new(addr); // 演示配置；不实际监听，示例即退出

    host.unload(&InstanceId::new("notebook").unwrap()).await?;
    Ok(())
}
