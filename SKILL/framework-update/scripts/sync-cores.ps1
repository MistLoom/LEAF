<#
.SYNOPSIS
  Sync Leaf framework cores into a downstream project (dry-run by default).
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string]$Source,
  [Parameter(Mandatory = $true)]
  [string]$Target,
  [switch]$Apply,
  [switch]$WithMonitor
)

$ErrorActionPreference = "Stop"

Add-Type -TypeDefinition @"
using System;
using System.IO;
using System.Security.Cryptography;
public static class FrameworkSyncHash {
  public static string Sha256Raw(string path) {
    using (var fs = File.OpenRead(path))
    using (var sha = SHA256.Create()) {
      return BitConverter.ToString(sha.ComputeHash(fs)).Replace("-", "");
    }
  }
  public static string Sha256Lf(string path) {
    byte[] bytes = File.ReadAllBytes(path);
    int n = 0;
    for (int i = 0; i < bytes.Length; i++) if (bytes[i] != 13) n++;
    byte[] outb = new byte[n];
    int j = 0;
    for (int i = 0; i < bytes.Length; i++) if (bytes[i] != 13) outb[j++] = bytes[i];
    using (var sha = SHA256.Create()) {
      return BitConverter.ToString(sha.ComputeHash(outb)).Replace("-", "");
    }
  }
}
"@

function Resolve-ExistingDir([string]$Path, [string]$Label) {
  if (-not (Test-Path -LiteralPath $Path)) { throw "$Label not found: $Path" }
  return (Resolve-Path -LiteralPath $Path).Path
}

$Source = Resolve-ExistingDir $Source "Source"
$Target = Resolve-ExistingDir $Target "Target"

$AlwaysCores = @("frame", "SDK", "apps/ability-core", "apps/plugin-host")
$OptionalMonitor = "apps/monitor"
$SkipDirNames = [System.Collections.Generic.HashSet[string]]::new(
  [string[]]@("target", ".git", "node_modules", "__pycache__", ".atomcode", ".cursor"),
  [StringComparer]::OrdinalIgnoreCase
)

function Test-SkipDir([string]$Name) {
  if ($SkipDirNames.Contains($Name)) { return $true }
  if ($Name -like "target-*") { return $true }
  return $false
}

function Get-RelativeFiles([string]$Root) {
  $map = @{}
  if (-not (Test-Path -LiteralPath $Root)) { return $map }
  $rootFull = (Resolve-Path -LiteralPath $Root).Path
  Get-ChildItem -LiteralPath $Root -Recurse -File -Force | ForEach-Object {
    $dir = $_.Directory
    $skip = $false
    while ($null -ne $dir -and $dir.FullName.Length -ge $rootFull.Length) {
      if (Test-SkipDir $dir.Name) { $skip = $true; break }
      if ($dir.FullName -eq $rootFull) { break }
      $dir = $dir.Parent
    }
    if ($skip) { return }
    $rel = $_.FullName.Substring($rootFull.Length).TrimStart("\", "/").Replace("\", "/")
    $map[$rel] = @{
      FullPath = $_.FullName
      Hash     = [FrameworkSyncHash]::Sha256Raw($_.FullName)
      NormHash = $null
      Length   = $_.Length
    }
  }
  return $map
}

function Ensure-NormHash($entry) {
  if ($null -eq $entry.NormHash) {
    $entry.NormHash = [FrameworkSyncHash]::Sha256Lf($entry.FullPath)
  }
  return $entry.NormHash
}

function Find-BuiltinMarkers([string]$CoreRoot, [string]$Label) {
  $hits = @()
  if (-not (Test-Path -LiteralPath $CoreRoot)) { return $hits }
  Get-ChildItem -LiteralPath $CoreRoot -Recurse -Filter "ability.yml" -File -ErrorAction SilentlyContinue | ForEach-Object {
    if ([System.IO.File]::ReadAllText($_.FullName) -match '(?m)^\s*leaf_builtin\s*:\s*true\s*$') {
      $hits += "ability.yml leaf_builtin: $($_.FullName)"
    }
  }
  Get-ChildItem -LiteralPath $CoreRoot -Recurse -Filter "LICENSE" -File -ErrorAction SilentlyContinue | ForEach-Object {
    if ([System.IO.File]::ReadAllText($_.FullName) -match 'LEAF-BUILTIN-APPS') {
      $hits += "LICENSE LEAF-BUILTIN-APPS: $($_.FullName)"
    }
  }
  if ($hits.Count -gt 0) {
    Write-Host "[builtin:$Label] $($hits.Count) marker(s) (read-only report)"
    $hits | Select-Object -First 20 | ForEach-Object { Write-Host "  $_" }
  }
  return $hits
}

function Compare-Core([string]$Rel) {
  $srcRoot = Join-Path $Source ($Rel -replace "/", [IO.Path]::DirectorySeparatorChar)
  $dstRoot = Join-Path $Target ($Rel -replace "/", [IO.Path]::DirectorySeparatorChar)
  $srcMap = Get-RelativeFiles $srcRoot
  $dstMap = Get-RelativeFiles $dstRoot
  $adds = New-Object System.Collections.Generic.List[string]
  $changes = New-Object System.Collections.Generic.List[string]
  $eols = New-Object System.Collections.Generic.List[string]
  $locals = New-Object System.Collections.Generic.List[string]
  $sames = 0
  foreach ($k in $srcMap.Keys) {
    if (-not $dstMap.ContainsKey($k)) { $adds.Add($k) | Out-Null; continue }
    if ($dstMap[$k].Hash -eq $srcMap[$k].Hash) { $sames++; continue }
    $sn = Ensure-NormHash $srcMap[$k]
    $dn = Ensure-NormHash $dstMap[$k]
    if ($sn -eq $dn) { $eols.Add($k) | Out-Null } else { $changes.Add($k) | Out-Null }
  }
  foreach ($k in $dstMap.Keys) {
    if (-not $srcMap.ContainsKey($k)) { $locals.Add($k) | Out-Null }
  }
  $addsArr = @($adds | Sort-Object)
  $changesArr = @($changes | Sort-Object)
  $eolsArr = @($eols | Sort-Object)
  $localsArr = @($locals | Sort-Object)
  return [pscustomobject]@{
    Rel = $Rel; SrcRoot = $srcRoot; DstRoot = $dstRoot
    SrcCount = $srcMap.Count; DstCount = $dstMap.Count; Same = $sames
    Adds = $addsArr; Changes = $changesArr; Eols = $eolsArr; Locals = $localsArr
    ConflictCount = $changesArr.Count + $localsArr.Count
    DiffCount = $addsArr.Count + $changesArr.Count + $localsArr.Count
  }
}

function Write-CoreReport($r, [string]$Mode) {
  Write-Host ""
  Write-Host "=== $($r.Rel) [$Mode] ==="
  Write-Host ("  source files={0}  target files={1}  same={2}  add={3}  change={4}  eol-only={5}  local-only={6}" -f `
      $r.SrcCount, $r.DstCount, $r.Same, $r.Adds.Count, $r.Changes.Count, $r.Eols.Count, $r.Locals.Count)
  $show = 12
  if ($r.Adds.Count -gt 0) {
    Write-Host "  [would-add]"
    $r.Adds | Select-Object -First $show | ForEach-Object { Write-Host "    + $_" }
    if ($r.Adds.Count -gt $show) { Write-Host ("    ... +{0} more" -f ($r.Adds.Count - $show)) }
  }
  if ($r.Changes.Count -gt 0) {
    Write-Host "  [conflict: content differs]"
    $r.Changes | Select-Object -First $show | ForEach-Object { Write-Host "    ~ $_" }
    if ($r.Changes.Count -gt $show) { Write-Host ("    ... +{0} more" -f ($r.Changes.Count - $show)) }
  }
  if ($r.Eols.Count -gt 0) {
    Write-Host "  [eol-only: CR/LF differs, content same after normalize]"
    $r.Eols | Select-Object -First 6 | ForEach-Object { Write-Host "    = $_" }
    if ($r.Eols.Count -gt 6) { Write-Host ("    ... +{0} more" -f ($r.Eols.Count - 6)) }
  }
  if ($r.Locals.Count -gt 0) {
    Write-Host "  [conflict: local-only in target; replace would remove]"
    $r.Locals | Select-Object -First $show | ForEach-Object { Write-Host "    - $_" }
    if ($r.Locals.Count -gt $show) { Write-Host ("    ... +{0} more" -f ($r.Locals.Count - $show)) }
  }
  if ($r.DiffCount -eq 0 -and $r.Eols.Count -eq 0) { Write-Host "  aligned" }
  elseif ($r.DiffCount -eq 0) { Write-Host "  aligned (content); eol-only only" }
}

function Backup-Conflicts($r, [string]$BakRoot) {
  $coreBak = Join-Path $BakRoot ($r.Rel -replace "/", [IO.Path]::DirectorySeparatorChar)
  $toBak = @($r.Changes + $r.Locals + $r.Eols | Select-Object -Unique)
  foreach ($rel in $toBak) {
    $srcFile = Join-Path $r.DstRoot ($rel -replace "/", [IO.Path]::DirectorySeparatorChar)
    if (-not (Test-Path -LiteralPath $srcFile)) { continue }
    $dstFile = Join-Path $coreBak ($rel -replace "/", [IO.Path]::DirectorySeparatorChar)
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $dstFile) | Out-Null
    Copy-Item -LiteralPath $srcFile -Destination $dstFile -Force
  }
  return $toBak.Count
}

function Apply-CoreReplace($r) {
  if (-not (Test-Path -LiteralPath $r.SrcRoot)) { throw "Source core missing: $($r.SrcRoot)" }
  $parent = Split-Path -Parent $r.DstRoot
  if ($parent -and -not (Test-Path -LiteralPath $parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }
  if (Test-Path -LiteralPath $r.DstRoot) { Remove-Item -LiteralPath $r.DstRoot -Recurse -Force }
  Copy-Item -LiteralPath $r.SrcRoot -Destination $r.DstRoot -Recurse -Force
}

$syncSet = @($AlwaysCores)
if ($WithMonitor) { $syncSet += $OptionalMonitor }

Write-Host "framework-update sync"
Write-Host "  Source : $Source"
Write-Host "  Target : $Target"
Write-Host "  Mode   : $(if ($Apply) { 'APPLY' } else { 'DRY-RUN' })"
Write-Host "  Cores  : $($syncSet -join ', ')"
if (-not $WithMonitor) { Write-Host "  Monitor: skipped (pass -WithMonitor to include)" }

$reports = @()
foreach ($rel in $syncSet) { $reports += (Compare-Core $rel) }

if (-not $WithMonitor) {
  $monSrc = Join-Path $Source ($OptionalMonitor -replace "/", [IO.Path]::DirectorySeparatorChar)
  $monDst = Join-Path $Target ($OptionalMonitor -replace "/", [IO.Path]::DirectorySeparatorChar)
  if ((Test-Path -LiteralPath $monSrc) -or (Test-Path -LiteralPath $monDst)) {
    Write-CoreReport (Compare-Core $OptionalMonitor) "skipped (no -WithMonitor)"
  }
}

$totalDiff = 0; $totalConflict = 0; $totalEol = 0; $totalAdd = 0
foreach ($r in $reports) {
  Write-CoreReport $r $(if ($Apply) { "apply-set" } else { "dry-run" })
  Find-BuiltinMarkers $r.SrcRoot "source/$($r.Rel)" | Out-Null
  Find-BuiltinMarkers $r.DstRoot "target/$($r.Rel)" | Out-Null
  $totalDiff += $r.DiffCount
  $totalConflict += $r.ConflictCount
  $totalEol += $r.Eols.Count
  $totalAdd += $r.Adds.Count
}

Write-Host ""
Write-Host "SUMMARY: content-diffs+adds+local=$totalDiff  conflicts(change+local-only)=$totalConflict  eol-only=$totalEol  adds=$totalAdd  cores=$($reports.Count)"

if (-not $Apply) {
  if ($totalDiff -eq 0) {
    if ($totalEol -gt 0) { Write-Host "DRY-RUN: content aligned; $totalEol eol-only file(s). No -Apply needed for semantics." }
    else { Write-Host "DRY-RUN: cores aligned; nothing to apply." }
    exit 0
  }
  Write-Host "DRY-RUN: differences found. Re-run with -Apply to overwrite after backup."
  Write-Host "  Backup path on apply: <Target>\.atomcode\framework-update-bak\<timestamp>\"
  if ($totalConflict -gt 0) { Write-Host "  Conflicts require explicit -Apply (no silent overwrite)." }
  exit 1
}

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$bakRoot = Join-Path $Target (Join-Path ".atomcode" (Join-Path "framework-update-bak" $stamp))
New-Item -ItemType Directory -Force -Path $bakRoot | Out-Null
Write-Host "Backup root: $bakRoot"
$backed = 0
foreach ($r in $reports) { $backed += Backup-Conflicts $r $bakRoot }
Write-Host "Backed up $backed conflicting/local-only/eol file(s)."
foreach ($r in $reports) { Write-Host "Replacing $($r.Rel) ..."; Apply-CoreReplace $r }

$postDiff = 0
foreach ($rel in $syncSet) {
  $r2 = Compare-Core $rel
  $postDiff += $r2.DiffCount + $r2.Eols.Count
  if ($r2.DiffCount -ne 0 -or $r2.Eols.Count -ne 0) { Write-CoreReport $r2 "post-apply" }
}
if ($postDiff -ne 0) { Write-Error "APPLY incomplete: residual diffs=$postDiff (backup at $bakRoot)"; exit 2 }
Write-Host "APPLY ok: cores replaced; residual diffs=0; backup=$bakRoot"
exit 0
