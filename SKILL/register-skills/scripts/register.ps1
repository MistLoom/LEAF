param(
  [ValidateSet("Project", "Global")]
  [string]$Scope = "Project",
  [string]$ProjectRoot = ""
)

$ErrorActionPreference = "Stop"
$skillRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$repoRoot = (Resolve-Path (Join-Path $skillRoot "..")).Path

$dirs = @(Get-ChildItem $skillRoot -Directory | Where-Object { Test-Path (Join-Path $_.FullName "SKILL.md") })
if ($dirs.Count -eq 0) { throw "No SKILL.md under $skillRoot" }

function Test-SkillDir([string]$dir) {
  $skill = Join-Path $dir "SKILL.md"
  if (-not (Test-Path $skill)) { throw "Missing SKILL.md in $dir" }
  $text = [System.IO.File]::ReadAllText($skill)
  if ($text -notmatch '(?m)^name:\s*\S+') { throw "SKILL.md missing name: $dir" }
  if ($text -notmatch '(?m)^description:') { throw "SKILL.md missing description: $dir" }
}

foreach ($d in $dirs) { Test-SkillDir $d.FullName }

if ($Scope -eq "Project") {
  if (-not $ProjectRoot) { $ProjectRoot = $repoRoot }
  $dstRoot = Join-Path $ProjectRoot ".cursor\skills"
} else {
  $dstRoot = Join-Path $HOME ".cursor\skills"
}

New-Item -ItemType Directory -Force -Path $dstRoot | Out-Null
foreach ($d in $dirs) {
  $target = Join-Path $dstRoot $d.Name
  if (Test-Path $target) { Remove-Item -Recurse -Force $target }
  Copy-Item -Recurse $d.FullName $target
  Test-SkillDir $target
  "registered $($d.Name) -> $target"
}
