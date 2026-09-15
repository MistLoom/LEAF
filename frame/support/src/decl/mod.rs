//! Ability 声明文件解析。运行时只读，与 codegen 共享 schema。

mod ability;
mod lint;
mod load;
mod package;
mod protocol;
mod version;

pub use ability::{AbilityDecl, MethodDecl, OutboundTopicDeclYaml};
pub use lint::{
    assert_clean_reports, discover_ability_roots, lint_ability_package, lint_workspace,
    LintOptions, LintReport,
};
pub use load::{
    detect_protocols, load_ability_decl, load_ability_package, load_cli_info, load_grpc_info,
    load_mcp_info, load_skill_info, load_tools_info,
};
pub use package::AbilityPackageMeta;
pub use protocol::{
    CliArgDecl, CliInfo, GrpcInfo, GrpcRpcDecl, McpInfo, McpToolDecl, ProtocolInfo, ProtocolKind,
    SkillInfo, ToolsInfo, ToolsMethodDecl,
};
pub use version::{validate_framework_version, FRAMEWORK_VERSION};