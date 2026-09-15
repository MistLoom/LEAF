//! 能力脚手架。

use std::fs;
use std::path::Path;

use ability_support::{Error, FRAMEWORK_VERSION};

use crate::ScaffoldArgs;

pub fn scaffold(args: &ScaffoldArgs) -> Result<(), Error> {
    let out = Path::new(&args.out);
    fs::create_dir_all(out.join("src").join("logic")).map_err(|e| Error::io(e.to_string()))?;

    let template = match args.template.as_str() {
        "minimal" | "with-cli" | "with-mcp" | "with-grpc" | "with-skill" => &args.template,
        other => {
            return Err(Error::invalid_args(format!("未知模板: {other}")));
        }
    };

    let ability_yml = include_str!("../templates/minimal/ability.yml")
        .replace("{{id}}", &args.id)
        .replace("{{framework_version}}", FRAMEWORK_VERSION);
    fs::write(out.join("ability.yml"), ability_yml).map_err(|e| Error::io(e.to_string()))?;

    fs::write(
        out.join("build.rs"),
        r#"fn main() {
    if let Err(e) = ability_sdk_codegen::generate_default() {
        panic!("ability codegen: {e}");
    }
}
"#,
    )
    .map_err(|e| Error::io(e.to_string()))?;

    fs::write(
        out.join("src").join("lib.rs"),
        r#"mod generated;
mod logic;

pub use generated::provider::provider;
"#,
    )
    .map_err(|e| Error::io(e.to_string()))?;

    for proto in &args.protocols {
        let dir = out.join(proto);
        fs::create_dir_all(&dir).map_err(|e| Error::io(e.to_string()))?;
        let info = match proto.as_str() {
            "cli" => include_str!("../templates/with-cli/cli/info.yml"),
            "mcp" => include_str!("../templates/with-mcp/mcp/info.yml"),
            "grpc" => include_str!("../templates/with-grpc/grpc/info.yml"),
            "skill" => include_str!("../templates/with-skill/skill/info.yml"),
            _ => continue,
        };
        fs::write(dir.join("info.yml"), info).map_err(|e| Error::io(e.to_string()))?;
    }

    let _ = template;
    Ok(())
}
