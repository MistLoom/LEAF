# 拉取市面生态依赖（Anthropic 官方 Mod 仓库 + DSH marketplace npm）
# 用法: 在仓库根目录执行 apps/web-agent/scripts/fetch-ecosystem-vendor.ps1

$ErrorActionPreference = "Stop"
$WebAgentDir = Split-Path -Parent $PSScriptRoot
$RepoRoot = Split-Path -Parent (Split-Path -Parent $WebAgentDir)
$Vendor = Join-Path $WebAgentDir "vendor"

function Ensure-GitClone {
    param(
        [string]$Url,
        [string]$Dest
    )
    if (Test-Path (Join-Path $Dest ".git")) {
        Write-Host "已存在，拉取最新: $Dest"
        Push-Location $Dest
        git pull --ff-only
        Pop-Location
        return
    }
    if (Test-Path $Dest) {
        throw "路径已存在且非 git 仓库: $Dest"
    }
    Write-Host "克隆: $Url -> $Dest"
    git clone --depth 1 $Url $Dest
}

New-Item -ItemType Directory -Force -Path $Vendor | Out-Null

Ensure-GitClone "https://github.com/anthropics/claude-plugins-official.git" (Join-Path $Vendor "claude-plugins-official")

# 可选参考仓库（体积较大，默认浅克隆）
$Optional = @(
    @{ Url = "https://github.com/anthropics/claude-plugins-community.git"; Dest = "claude-plugins-community" },
    @{ Url = "https://github.com/ydhrdh/dsh-marketplace.git"; Dest = "dsh-marketplace" }
)
foreach ($repo in $Optional) {
    $dest = Join-Path $Vendor $repo.Dest
    try {
        Ensure-GitClone $repo.Url $dest
    } catch {
        Write-Warning "跳过可选仓库 $($repo.Dest): $_"
    }
}

$DshProfile = Join-Path $Vendor "dsh-npm-profile"
if (-not (Test-Path (Join-Path $DshProfile "package.json"))) {
    throw "缺少 $DshProfile/package.json"
}

Write-Host "npm install: $DshProfile"
Push-Location $DshProfile
if (Test-Path "package-lock.json") {
    npm ci
} else {
    npm install
}
Pop-Location

Write-Host "完成。在仓库根目录 $RepoRoot 执行:"
Write-Host "  cargo run -p web-agent -- --plugins apps/web-agent/market/market.plugins.yaml doctor"
