//! 能力脚手架。

use std::fs;
use std::path::Path;

use ability_support::{Error, FRAMEWORK_VERSION};

use crate::generated::handlers_trait::ScaffoldArgs;

pub fn scaffold(args: &ScaffoldArgs) -> Result<(), Error> {
    let out = Path::new(&args.out);

    let template = match args.template.as_str() {
        "minimal" | "with-cli" | "with-mcp" | "with-grpc" | "with-skill" | "minimal-node"
        | "minimal-python" => &args.template,
        other => {
            return Err(Error::invalid_args(format!("未知模板: {other}")));
        }
    };

    match template.as_str() {
        "minimal-node" => scaffold_node(out, args)?,
        "minimal-python" => scaffold_python(out, args)?,
        _ => scaffold_rust(out, args, template)?,
    }
    Ok(())
}

fn scaffold_rust(out: &Path, args: &ScaffoldArgs, template: &str) -> Result<(), Error> {
    fs::create_dir_all(out.join("src").join("logic")).map_err(|e| Error::io(e.to_string()))?;

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
        write_protocol_info(out, proto)?;
    }
    let _ = template;
    Ok(())
}

fn scaffold_node(out: &Path, args: &ScaffoldArgs) -> Result<(), Error> {
    fs::create_dir_all(out.join("src")).map_err(|e| Error::io(e.to_string()))?;
    let ability_yml =
        include_str!("../templates/minimal-node/ability.yml").replace("{{id}}", &args.id);
    fs::write(out.join("ability.yml"), ability_yml).map_err(|e| Error::io(e.to_string()))?;
    fs::write(
        out.join("package.json"),
        r#"{
  "name": "{{id}}",
  "version": "0.1.0",
  "type": "module",
  "private": true
}
"#
        .replace("{{id}}", &args.id),
    )
    .map_err(|e| Error::io(e.to_string()))?;
    let serve = include_str!("../templates/minimal-node/src/serve.mjs");
    fs::write(out.join("src").join("serve.mjs"), serve).map_err(|e| Error::io(e.to_string()))?;
    for proto in &args.protocols {
        write_protocol_info(out, proto)?;
    }
    Ok(())
}

fn scaffold_python(out: &Path, args: &ScaffoldArgs) -> Result<(), Error> {
    fs::create_dir_all(out.join("src")).map_err(|e| Error::io(e.to_string()))?;
    let ability_yml =
        include_str!("../templates/minimal-python/ability.yml").replace("{{id}}", &args.id);
    fs::write(out.join("ability.yml"), ability_yml).map_err(|e| Error::io(e.to_string()))?;
    fs::write(
        out.join("pyproject.toml"),
        format!(
            r#"[project]
name = "{id}"
version = "0.1.0"
requires-python = ">=3.11"

[build-system]
requires = ["setuptools>=61"]
build-backend = "setuptools.build_meta"

[tool.setuptools.packages.find]
where = ["src"]
"#,
            id = args.id
        ),
    )
    .map_err(|e| Error::io(e.to_string()))?;
    let serve = include_str!("../templates/minimal-python/src/serve.py");
    fs::write(out.join("src").join("serve.py"), serve).map_err(|e| Error::io(e.to_string()))?;
    for proto in &args.protocols {
        write_protocol_info(out, proto)?;
    }
    Ok(())
}

fn write_protocol_info(out: &Path, proto: &str) -> Result<(), Error> {
    let dir = out.join(proto);
    fs::create_dir_all(&dir).map_err(|e| Error::io(e.to_string()))?;
    let info = match proto {
        "cli" => include_str!("../templates/with-cli/cli/info.yml"),
        "mcp" => include_str!("../templates/with-mcp/mcp/info.yml"),
        "grpc" => include_str!("../templates/with-grpc/grpc/info.yml"),
        "skill" => include_str!("../templates/with-skill/skill/info.yml"),
        _ => return Ok(()),
    };
    fs::write(dir.join("info.yml"), info).map_err(|e| Error::io(e.to_string()))?;
    Ok(())
}
