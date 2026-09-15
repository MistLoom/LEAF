//! 生成 handlers_trait.rs。

use ability_support::AbilityPackageMeta;

pub fn generate(meta: &AbilityPackageMeta) -> Result<String, String> {
    let mut methods = String::new();
    let mut args_structs = String::new();

    for m in &meta.decl.methods {
        let fn_name = method_fn_name(&m.id);
        let struct_name = method_args_name(&m.id);

        args_structs.push_str(&format!(
            r#"
#[derive(Debug, serde::Deserialize)]
pub struct {struct_name} {{}}
"#,
            struct_name = struct_name,
        ));

        methods.push_str(&format!(
            r#"
    async fn {fn_name}(&self, ctx: &dyn ability_support::AbilityCtx, args: serde_json::Value) -> ability_support::Result<serde_json::Value>;
"#,
            fn_name = fn_name,
        ));
    }

    Ok(format!(
        r#"// 自动生成，勿手改
use async_trait::async_trait;

{args_structs}

#[async_trait]
pub trait Handlers: Send + Sync {{
{methods}
}}
"#,
        args_structs = args_structs,
        methods = methods,
    ))
}

pub fn method_fn_name(id: &str) -> String {
    id.replace('.', "_").replace('-', "_")
}

pub fn method_args_name(id: &str) -> String {
    let base = method_fn_name(id);
    let mut chars = base.chars();
    match chars.next() {
        None => "Args".into(),
        Some(c) => c.to_uppercase().chain(chars).collect::<String>() + "Args",
    }
}
