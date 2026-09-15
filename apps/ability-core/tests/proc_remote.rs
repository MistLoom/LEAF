//! 远程 Provider：duplex 协议、跨进程回呼、unload 后再接新会话。

use std::sync::Arc;

use ability_sdk::prelude::*;
use ability_sdk::{serve, AbilityProvider, AbilitySpec};
use ability_support::{
    EventClass, EventFilter, EventListener, InstanceId, LoadSpec, MethodId, SourceId, StaticProvider,
};
use frame_core::InvokeRequest;
use ability_core::{connect, AbilityHost};
use serde_json::json;

fn sid(s: &str) -> SourceId {
    SourceId::new(s).unwrap()
}

fn iid(s: &str) -> InstanceId {
    InstanceId::new(s).unwrap()
}

fn mid(s: &str) -> MethodId {
    MethodId::new(s).unwrap()
}

async fn install_connected(
    host: &AbilityHost,
    source: &str,
    ability: Arc<dyn ability_support::Ability>,
) {
    host.install(sid(source), Arc::new(StaticProvider::new(ability)))
        .await;
    host.load(LoadSpec::new(sid(source), iid(source)))
        .await
        .unwrap();
}

fn stamp_provider(stamp: &'static str) -> Arc<dyn AbilityProvider> {
    AbilitySpec::must("stamp")
        .on(
            "run",
            "印记",
            json!({ "type": "object" }),
            handler!(|_ctx, _args| Ok(json!({ "stamp": stamp }))),
        )
        .into_provider()
}

#[tokio::test(flavor = "multi_thread")]
async fn duplex_probe_invoke_emit_unload() {
    let (client, server) = tokio::io::duplex(64 * 1024);
    let (server_r, server_w) = tokio::io::split(server);
    let (client_r, client_w) = tokio::io::split(client);
    tokio::spawn(async move {
        serve(ability_probe::provider(), server_r, server_w)
            .await
            .unwrap();
    });
    let ability = connect(client_r, client_w, json!({})).await.unwrap();

    let host = AbilityHost::new();
    install_connected(&host, "probe", ability).await;

    let mut sub = host
        .events()
        .listen(EventFilter {
            class: Some(EventClass::Ability),
            name: Some("pong".into()),
            instance: None,
        })
        .await;

    let gw = host.clone().into_gateway();
    let resp = gw
        .invoke(InvokeRequest::new(iid("probe"), mid("ping"), json!({})))
        .await
        .unwrap();
    assert_eq!(resp.data["ok"], true);
    assert_eq!(resp.data["from"], "external");

    let ev = sub.next().await.expect("pong");
    assert_eq!(ev.name(), "pong");

    host.unload(&iid("probe")).await.unwrap();
    let err = gw
        .invoke(InvokeRequest::new(iid("probe"), mid("ping"), json!({})))
        .await
        .unwrap_err();
    assert_eq!(err.kind(), ability_support::ErrorKind::NotFound);
}

#[tokio::test(flavor = "multi_thread")]
async fn duplex_wrap_calls_in_process_echo() {
    let (client, server) = tokio::io::duplex(64 * 1024);
    let (server_r, server_w) = tokio::io::split(server);
    let (client_r, client_w) = tokio::io::split(client);
    tokio::spawn(async move {
        serve(ability_probe::wrap_provider(), server_r, server_w)
            .await
            .unwrap();
    });
    let wrap = connect(client_r, client_w, json!({})).await.unwrap();

    let host = AbilityHost::new();
    host.install(sid("probe.echo"), ability_probe::echo_provider())
        .await;
    host.load(LoadSpec::new(sid("probe.echo"), iid("probe.echo")))
        .await
        .unwrap();
    install_connected(&host, "probe.wrap", wrap).await;

    let resp = host
        .clone()
        .into_gateway()
        .invoke(InvokeRequest::new(
            iid("probe.wrap"),
            mid("run"),
            json!({ "n": 2 }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.data["wrapped"]["n"], 2);
    assert_eq!(resp.data["fx"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn duplex_unload_reload_new_remote() {
    async fn boot(stamp: &'static str) -> Arc<dyn ability_support::Ability> {
        let (client, server) = tokio::io::duplex(64 * 1024);
        let (server_r, server_w) = tokio::io::split(server);
        let (client_r, client_w) = tokio::io::split(client);
        tokio::spawn(async move {
            serve(stamp_provider(stamp), server_r, server_w)
                .await
                .unwrap();
        });
        connect(client_r, client_w, json!({})).await.unwrap()
    }

    let host = AbilityHost::new();
    install_connected(&host, "stamp", boot("a").await).await;
    let gw = host.clone().into_gateway();
    let a = gw
        .invoke(InvokeRequest::new(iid("stamp"), mid("run"), json!({})))
        .await
        .unwrap();
    assert_eq!(a.data["stamp"], "a");

    host.unload(&iid("stamp")).await.unwrap();
    install_connected(&host, "stamp", boot("b").await).await;
    let b = gw
        .invoke(InvokeRequest::new(iid("stamp"), mid("run"), json!({})))
        .await
        .unwrap();
    assert_eq!(b.data["stamp"], "b");
}
