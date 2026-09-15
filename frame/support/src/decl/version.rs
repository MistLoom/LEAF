//! 声明文件框架版本。codegen 与运行时解析共用。

/// 当前声明 schema 版本。`ability.yml` 与各协议 `info.yml` 须与此一致。
pub const FRAMEWORK_VERSION: &str = "1";

/// 校验声明中的 `framework_version`。
pub fn validate_framework_version(file: &str, declared: Option<&str>) -> Result<(), String> {
    match declared {
        None => Err(format!(
            "{file} 缺少 framework_version（当前要求 {FRAMEWORK_VERSION}）"
        )),
        Some(v) if v == FRAMEWORK_VERSION => Ok(()),
        Some(v) => Err(format!(
            "{file} framework_version 不兼容：声明 {v}，当前 {FRAMEWORK_VERSION}"
        )),
    }
}
