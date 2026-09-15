$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$cfgRel = 'config/tool_expose.example.yaml'
$cfg = Join-Path $root $cfgRel
$out = Join-Path $PSScriptRoot 'TEST_RESULTS.md'
$showcase = Join-Path $root 'external\showcase'
$results = New-Object System.Collections.Generic.List[object]
$sections = New-Object System.Collections.Generic.List[string]

function Invoke-Step {
    param(
        [string]$Name,
        [string]$WorkDir,
        [scriptblock]$Action,
        [string]$CmdDisplay,
        [bool]$ExpectFail = $false
    )
    $sections.Add('')
    $sections.Add("## $Name")
    $sections.Add('')
    $sections.Add("- cwd: ``$WorkDir``")
    $sections.Add("- cmd: ``$CmdDisplay``")
    $sections.Add("- expect_fail: $ExpectFail")
    $sections.Add('')
    Push-Location $WorkDir
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & $Action 2>&1 | ForEach-Object { $_.ToString() }
    $code = $LASTEXITCODE
    if ($null -eq $code) { $code = 0 }
    $sw.Stop()
    Pop-Location
    $ok = if ($ExpectFail) { $code -ne 0 } else { $code -eq 0 }
    $status = if ($ok) { if ($ExpectFail) { 'PASS(expected_fail)' } else { 'PASS' } } else { 'FAIL' }
    $sections.Add("- result: **$status** (exit=$code, $($sw.ElapsedMilliseconds)ms)")
    $sections.Add('')
    $sections.Add('```')
    $body = ($output -join "`n").Trim()
    if ([string]::IsNullOrWhiteSpace($body)) { $body = '(no output)' }
    $sections.Add($body)
    $sections.Add('```')
    $sections.Add('')
    $results.Add([pscustomobject]@{ Name = $Name; Status = $status; Code = $code; Ok = $ok; Body = $body })
}

$sections.Add('# leafEX real config and third-party access test')
$sections.Add('')
$sections.Add("- time: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss K')")
$sections.Add("- config: ``$cfgRel``")
$sections.Add("- config_abs: ``$cfg``")
$sections.Add("- third_party: ``external/showcase``")
$sections.Add('')
if (-not (Test-Path $cfg)) { throw "config missing: $cfg" }

Invoke-Step -Name '1. build apps-runtime' -WorkDir $root -CmdDisplay 'cargo build -p apps-runtime' -Action { cargo build -p apps-runtime }
Invoke-Step -Name '2. build ability CLIs' -WorkDir $root -CmdDisplay 'cargo run -q -p apps-runtime --bin tool-bin -- build' -Action { cargo run -q -p apps-runtime --bin tool-bin -- build }
Invoke-Step -Name '3. build third-party showcase' -WorkDir $showcase -CmdDisplay 'cargo build' -Action { cargo build }
Invoke-Step -Name '4. CLI catalog list with real expose' -WorkDir $root -CmdDisplay "cargo run -q -p apps-runtime -- --config $cfgRel catalog list" -Action { cargo run -q -p apps-runtime -- --config $cfgRel catalog list }

# Windows PowerShell: keep JSON quotes via backslash escapes inside single quotes
Invoke-Step -Name '5. CLI invoke read_file (allowed)' -WorkDir $root -CmdDisplay 'cargo run -q -p apps-runtime -- --config ... invoke read_file --args {\"path\":\"README.md\"}' -Action { cargo run -q -p apps-runtime -- --config $cfgRel invoke read_file --args '{\"path\":\"README.md\"}' }

Invoke-Step -Name '6. third-party showcase full run' -WorkDir $showcase -CmdDisplay 'cargo run --quiet' -Action { cargo run --quiet }

$pass = @($results | Where-Object { $_.Ok }).Count
$fail = @($results | Where-Object { -not $_.Ok }).Count
$overview = New-Object System.Collections.Generic.List[string]
$overview.Add('## Summary')
$overview.Add('')
$overview.Add("- PASS: $pass / $($results.Count)")
$overview.Add("- FAIL: $fail / $($results.Count)")
foreach ($r in $results) { $overview.Add("- $($r.Status) - $($r.Name)") }
$overview.Add('')
$overview.Add('## Notes')
$overview.Add('')
$overview.Add('- Step 5: read_file enabled in tool_expose.example.yaml.')
$overview.Add('- Step 6: external/showcase consumes framework example expose; no local expose.yaml.')
$overview.Add('')

$final = New-Object System.Collections.Generic.List[string]
$headerEnd = 6
for ($i = 0; $i -lt $headerEnd; $i++) { $final.Add($sections[$i]) }
$final.AddRange($overview)
for ($i = $headerEnd; $i -lt $sections.Count; $i++) { $final.Add($sections[$i]) }
$utf8 = New-Object System.Text.UTF8Encoding $true
[System.IO.File]::WriteAllLines($out, $final.ToArray(), $utf8)
Write-Host "WROTE $out"
Write-Host "PASS=$pass FAIL=$fail"
$results | ForEach-Object { Write-Host ("{0} - {1}" -f $_.Status, $_.Name) }
if ($fail -gt 0) { exit 1 } else { exit 0 }
