use std::time::Duration;

use ability_probe::{
    TraceAspect, echo_provider, hub_provider, note_provider, provider, wrap_provider,
};
use ability_support::{
    EventClass, EventFilter, EventListener, InstanceId, LoadSpec, MethodId, SourceId,
};
use frame_core::InvokeRequest;
use ability_core::AbilityHost;
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

async fn wait_status_has(gw: &dyn frame_core::Gateway, pred: impl Fn(&serde_json::Value) -> bool) -> serde_json::Value {
    let mut last = json!({});
    for _ in 0..50 {
        last = gw
            .invoke(InvokeRequest::new(iid("probe.hub"), mid("status"), json!({})))
            .await
            .unwrap()
            .data;
        if pred(&last) {
            return last;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    last
}

#[tokio::test]
async fn external_probe_install_invoke_unload() {
    let host = AbilityHost::new();
    host.install(sid("probe"), provider()).await;
    host.load(LoadSpec::new(sid("probe"), iid("probe")))
        .await
        .unwrap();

    let mut sub = host
        .events()
        .listen(EventFilter {
            class: Some(EventClass::Ability),
            name: Some("pong".into()),
            instance: None,
        })
        .await;

    let resp = host
        .clone()
        .into_gateway()
        .invoke(InvokeRequest::new(iid("probe"), mid("ping"), json!({})))
        .await
        .unwrap();
    assert_eq!(resp.data["from"], "external");
    assert_eq!(resp.data["ok"], true);

    let ev = sub.next().await.expect("pong");
    assert_eq!(ev.name(), "pong");

    host.unload(&iid("probe")).await.unwrap();
    assert!(
        !host
            .catalog()
            .iter()
            .any(|e| e.ability_id.as_str() == "probe")
    );
}

#[tokio::test]
async fn external_wrap_calls_echo_via_alias_and_effect() {
    let host = AbilityHost::new();
    host.install(sid("probe.echo"), echo_provider()).await;
    host.install(sid("probe.wrap"), wrap_provider()).await;
    host.load(LoadSpec::new(sid("probe.echo"), iid("probe.echo")))
        .await
        .unwrap();
    host.load(LoadSpec::new(sid("probe.wrap"), iid("probe.wrap")))
        .await
        .unwrap();

    let mut sub = host
        .events()
        .listen(EventFilter {
            class: Some(EventClass::Ability),
            name: Some("done".into()),
            instance: None,
        })
        .await;

    let args = json!({ "text": "hello" });
    let resp = host
        .clone()
        .into_gateway()
        .invoke(InvokeRequest::new(
            iid("probe.wrap"),
            mid("run"),
            args.clone(),
        ))
        .await
        .unwrap();
    assert_eq!(resp.data["wrapped"], args);
    assert_eq!(resp.data["fx"], true);

    let ev = sub.next().await.expect("done");
    assert_eq!(ev.name(), "done");
}

#[tokio::test]
async fn external_hub_covers_support_surface() {
    let host = AbilityHost::new();
    let trace = TraceAspect::new();
    host.add_aspect(trace.clone());

    host.install(sid("probe.echo"), echo_provider()).await;
    host.install(sid("probe.wrap"), wrap_provider()).await;
    host.install(sid("probe.hub"), hub_provider()).await;
    host.install(sid("probe.note"), note_provider()).await;

    host.load(LoadSpec::new(sid("probe.echo"), iid("probe.echo")))
        .await
        .unwrap();

    let mut ready = host
        .events()
        .listen(EventFilter {
            class: Some(EventClass::Ability),
            name: Some("ready".into()),
            instance: None,
        })
        .await;
    let mut caught = host
        .events()
        .listen(EventFilter {
            class: Some(EventClass::Ability),
            name: Some("caught".into()),
            instance: None,
        })
        .await;

    host.load(
        LoadSpec::new(sid("probe.hub"), iid("probe.hub")).with_config(json!({ "tag": "hub-1" })),
    )
    .await
    .unwrap();
    assert_eq!(ready.next().await.expect("ready").name(), "ready");

    host.load(LoadSpec::new(sid("probe.wrap"), iid("probe.wrap")))
        .await
        .unwrap();

    let gw = host.clone().into_gateway();
    let args = json!({ "text": "hub" });
    gw.invoke(InvokeRequest::new(
        iid("probe.wrap"),
        mid("run"),
        args.clone(),
    ))
    .await
    .unwrap();
    assert_eq!(caught.next().await.expect("caught").name(), "caught");

    let st = wait_status_has(&*gw, |st| {
        st["heard"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("done")))
            && !st["injected"].as_array().map(|a| a.is_empty()).unwrap_or(true)
    })
    .await;
    assert_eq!(st["tag"], "hub-1");
    assert_eq!(st["echo"], "probe.echo");
    assert!(
        st["catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some("probe.hub"))
    );
    assert!(st["injected"].as_array().unwrap()[0]["injected"] == "done");

    gw.invoke(InvokeRequest::new(iid("probe.hub"), mid("boot"), json!({})))
        .await
        .unwrap();
    assert!(host.catalog().iter().any(|e| e.instance.as_str() == "probe.note"));

    gw.invoke(InvokeRequest::new(
        iid("probe.note"),
        mid("write"),
        json!({ "k": 1 }),
    ))
    .await
    .unwrap();
    assert_eq!(caught.next().await.expect("noted→caught").name(), "caught");

    let st = wait_status_has(&*gw, |st| {
        st["heard"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("noted")))
    })
    .await;
    assert!(st["catalog"].as_array().unwrap().iter().any(|v| v.as_str() == Some("probe.note")));

    gw.invoke(InvokeRequest::new(iid("probe.hub"), mid("unboot"), json!({})))
        .await
        .unwrap();
    assert!(
        !host
            .catalog()
            .iter()
            .any(|e| e.instance.as_str() == "probe.note")
    );

    let ping = gw
        .invoke(InvokeRequest::new(
            iid("probe.hub"),
            mid("ping"),
            json!({ "from": "hub" }),
        ))
        .await
        .unwrap();
    assert_eq!(ping.data["via_ext"]["from"], "hub");
    assert_eq!(ping.data["via_macro"]["from"], "hub");
    assert!(ping.data["aspect_hits"].as_u64().unwrap_or(0) >= 1);
    assert!(trace.hits() >= 1);

    host.unload(&iid("probe.hub")).await.unwrap();
    assert!(
        !host
            .catalog()
            .iter()
            .any(|e| e.ability_id.as_str() == "probe.hub")
    );
}
