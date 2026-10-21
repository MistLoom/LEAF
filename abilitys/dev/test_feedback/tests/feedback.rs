//! 经 `test.feedback` 覆盖下单扣款、无 ctx、非法标签、处理器、落盘重载。

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ability_core::AbilityHost;
use ability_support::{
    DETACHED_CHAIN, FeedbackProcessor, FeedbackRecord, InstanceId, LoadSpec, MethodId, SourceId,
    TagFilter,
};
use frame_core::InvokeRequest;
use serde_json::json;

struct Cap(Mutex<Vec<FeedbackRecord>>);

impl FeedbackProcessor for Cap {
    fn on_feedback(&self, record: &FeedbackRecord) {
        self.0.lock().unwrap().push(record.clone());
    }
}

struct Boom;

impl FeedbackProcessor for Boom {
    fn on_feedback(&self, _record: &FeedbackRecord) {
        panic!("processor boom");
    }
}

fn disk_records(path: &Path) -> Vec<FeedbackRecord> {
    let f = fs::File::open(path).expect("jsonl 应已落盘");
    BufReader::new(f)
        .lines()
        .map(|l| l.expect("读行"))
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(&l).expect("jsonl 行须为 FeedbackRecord"))
        .collect()
}

fn isolate() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn app_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/feedback-test")
}

fn events_path() -> PathBuf {
    std::env::current_dir()
        .expect("cwd")
        .join("data")
        .join("feedback")
        .join("events.jsonl")
}

fn enter_app() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let root = app_root();
        fs::create_dir_all(root.join("data").join("feedback")).unwrap();
        std::env::set_current_dir(&root).expect("进入应用根目录");
    });
}

fn setup() -> PathBuf {
    enter_app();
    ability_support::feedback::purge();
    events_path()
}

const SRC: &str = "test.feedback";

async fn boot() -> AbilityHost {
    let host = AbilityHost::new();
    host.install_str(SRC, ability_test_feedback::provider())
        .await
        .unwrap();
    host.load(LoadSpec::new(
        SourceId::new(SRC).unwrap(),
        InstanceId::new(SRC).unwrap(),
    ))
    .await
    .unwrap();
    host
}

async fn invoke(host: &AbilityHost, method: &str, args: serde_json::Value) {
    host.clone()
        .into_gateway()
        .invoke(InvokeRequest::new(
            InstanceId::new(SRC).unwrap(),
            MethodId::new(method).unwrap(),
            args,
        ))
        .await
        .unwrap();
}

#[tokio::test]
async fn order_pay_chain_spans_and_disk() {
    let _g = isolate();
    let path = setup();
    let host = boot().await;
    let gw = host.into_gateway();
    gw.invoke(InvokeRequest::new(
        InstanceId::new(SRC).unwrap(),
        MethodId::new("place").unwrap(),
        json!({ "amt": 42 }),
    ))
    .await
    .unwrap();

    let mem = ability_support::feedback::list();
    let begin = mem.iter().find(|r| r.desc == "begin").expect("begin");
    let charged = mem
        .iter()
        .find(|r| r.desc == "charged 42")
        .expect("charged");
    let done = mem.iter().find(|r| r.desc == "done").expect("done");
    assert_eq!(begin.chain_id, charged.chain_id);
    assert_eq!(begin.chain_id, done.chain_id);
    assert_ne!(begin.chain_id, DETACHED_CHAIN);
    assert_eq!(begin.span, 0);
    assert_eq!(charged.span, 1);
    assert_eq!(done.span, 0);
    assert_eq!(begin.tags, vec!["order"]);
    assert_eq!(charged.tags, vec!["pay"]);
    assert_eq!(begin.loc.ability, "ability-test-feedback");
    assert!(begin.loc.module.contains("ability_test_feedback"));

    let i1 = mem.iter().position(|r| r.desc == "begin").unwrap();
    let i2 = mem.iter().position(|r| r.desc == "charged 42").unwrap();
    let i3 = mem.iter().position(|r| r.desc == "done").unwrap();
    assert!(i1 < i2 && i2 < i3);

    let disk = disk_records(&path);
    assert_eq!(disk.len(), mem.len());
    assert_eq!(
        disk.iter().map(|r| &r.id).collect::<Vec<_>>(),
        mem.iter().map(|r| &r.id).collect::<Vec<_>>()
    );

    let by_chain: Vec<_> = mem
        .iter()
        .filter(|r| r.chain_id == begin.chain_id)
        .collect();
    assert_eq!(by_chain.len(), 3);
    assert_eq!(
        ability_support::feedback::get(&charged.id).unwrap().desc,
        "charged 42"
    );
    let pay_only = mem
        .iter()
        .filter(|r| r.tags.iter().any(|t| t == "pay"))
        .count();
    assert_eq!(pay_only, 1);
}

#[tokio::test]
async fn two_orders_isolated_chains() {
    let _g = isolate();
    let path = setup();
    let host = boot().await;
    let gw = host.into_gateway();
    gw.invoke(InvokeRequest::new(
        InstanceId::new(SRC).unwrap(),
        MethodId::new("place").unwrap(),
        json!({ "amt": 1 }),
    ))
    .await
    .unwrap();
    gw.invoke(InvokeRequest::new(
        InstanceId::new(SRC).unwrap(),
        MethodId::new("place").unwrap(),
        json!({ "amt": 2 }),
    ))
    .await
    .unwrap();

    let begins: Vec<_> = ability_support::feedback::list()
        .into_iter()
        .filter(|r| r.desc == "begin")
        .collect();
    assert_eq!(begins.len(), 2);
    assert_ne!(begins[0].chain_id, begins[1].chain_id);
    let listed = ability_support::feedback::list();
    assert_eq!(
        listed
            .iter()
            .filter(|r| r.chain_id == begins[0].chain_id)
            .count(),
        3
    );
    assert_eq!(
        listed
            .iter()
            .filter(|r| r.chain_id == begins[1].chain_id)
            .count(),
        3
    );
    let disk = disk_records(&path);
    assert_eq!(disk.iter().filter(|r| r.desc == "begin").count(), 2);
}

#[tokio::test]
async fn detached_and_invalid_tags_on_disk() {
    let _g = isolate();
    let path = setup();
    ability_test_feedback::emit_detached();
    let host = boot().await;
    invoke(&host, "mixed", json!({})).await;

    let disk = disk_records(&path);
    let bg = disk.iter().find(|r| r.desc == "background").unwrap();
    assert_eq!(bg.chain_id, DETACHED_CHAIN);
    assert_eq!(bg.span, 0);
    let mixed = disk.iter().find(|r| r.desc == "mixed").unwrap();
    assert_eq!(mixed.tags, vec!["ok", "x"]);
    assert!(!mixed.tags.iter().any(|t| t.contains('_')));
}

#[tokio::test]
async fn processors_filter_duplicate_and_panic_still_persist() {
    let _g = isolate();
    let path = setup();
    let host = boot().await;
    let all = Arc::new(Cap(Mutex::new(Vec::new())));
    let audit = Arc::new(Cap(Mutex::new(Vec::new())));
    let _boom = ability_support::feedback::register_processor(Arc::new(Boom), TagFilter::all());
    let _all1 =
        ability_support::feedback::register_processor(Arc::clone(&all) as _, TagFilter::all());
    let _all2 =
        ability_support::feedback::register_processor(Arc::clone(&all) as _, TagFilter::all());
    let _audit = ability_support::feedback::register_processor(
        Arc::clone(&audit) as _,
        TagFilter::only(vec!["audit".into()]),
    );

    invoke(&host, "note", json!({ "tags": ["audit"], "desc": "money" })).await;
    invoke(&host, "note", json!({ "tags": ["info"], "desc": "hello" })).await;

    assert_eq!(all.0.lock().unwrap().len(), 4);
    let audit_got = audit.0.lock().unwrap();
    assert_eq!(audit_got.len(), 1);
    assert_eq!(audit_got[0].desc, "money");

    let disk = disk_records(&path);
    assert!(disk.iter().any(|r| r.desc == "money"));
    assert!(disk.iter().any(|r| r.desc == "hello"));
}

#[tokio::test]
async fn reload_same_file_then_purge() {
    let _g = isolate();
    let path = setup();
    let host = boot().await;
    invoke(&host, "note", json!({ "tags": ["save"], "desc": "first" })).await;
    let id = ability_support::feedback::list()[0].id.clone();

    ability_support::feedback::host::bind(&path);
    let reloaded = ability_support::feedback::get(&id).expect("重载后仍可按 id 取");
    assert_eq!(reloaded.desc, "first");
    assert_eq!(disk_records(&path).len(), 1);

    invoke(&host, "note", json!({ "tags": ["tmp"], "desc": "gone" })).await;
    ability_support::feedback::purge();
    assert!(ability_support::feedback::list().is_empty());
    let body = fs::read_to_string(&path).unwrap();
    assert!(body.trim().is_empty());
}

#[tokio::test]
async fn sequential_notes_on_disk() {
    let _g = isolate();
    let path = setup();
    let host = boot().await;
    invoke(&host, "note", json!({ "tags": ["aa"], "desc": "one" })).await;
    invoke(&host, "note", json!({ "tags": ["bb"], "desc": "two" })).await;
    let all = ability_support::feedback::list();
    let i1 = all.iter().position(|r| r.desc == "one").unwrap();
    let i2 = all.iter().position(|r| r.desc == "two").unwrap();
    assert!(i1 < i2);
    let on_disk = disk_records(&path);
    assert_eq!(on_disk.len(), all.len());
    assert_eq!(on_disk[i1].desc, "one");
    assert_eq!(on_disk[i2].desc, "two");
}

#[tokio::test]
async fn accept_remote_writes_disk() {
    let _g = isolate();
    let path = setup();
    ability_support::feedback::host::accept(FeedbackRecord {
        id: "remote-1".into(),
        at: 1,
        loc: ability_support::ArchLoc {
            ability: "guest-pkg".into(),
            module: "guest::work".into(),
        },
        tags: vec!["net".into()],
        desc: "from guest".into(),
        chain_id: "chain-g".into(),
        span: 3,
    });
    let disk = disk_records(&path);
    assert_eq!(disk.len(), 1);
    assert_eq!(disk[0].id, "remote-1");
    assert_eq!(disk[0].loc.module, "guest::work");
    assert_eq!(disk[0].span, 3);
    assert_eq!(
        ability_support::feedback::list()
            .iter()
            .find(|r| r.chain_id == "chain-g")
            .unwrap()
            .desc,
        "from guest"
    );
}
