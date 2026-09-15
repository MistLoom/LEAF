//! 全仓库能力声明规范校验。

use ability_support::{assert_clean_reports, lint_workspace, LintOptions};
use frame_compose::workspace_root;

#[test]
fn all_ability_decls_pass_lint() {
    let workspace = workspace_root();
    let opts = LintOptions::workspace_strict();
    let reports = lint_workspace(&workspace, &opts).expect("lint workspace");
    assert_eq!(
        reports.len(),
        24,
        "应校验 24 个内置能力声明目录"
    );
    assert_clean_reports(&reports).expect("声明校验失败");
}
