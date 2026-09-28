$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$fixture = (Get-ChildItem "$root/dist/*.zip")[0].FullName
$env:HNM_TEST_VERSION = (Select-String '^version = "([^"]+)"' "$root/Cargo.toml").Matches[0].Groups[1].Value
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ("hnm-test-" + [guid]::NewGuid())
$oldPath = $env:PATH
$oldDest = $env:HNM_INSTALL_DIR
$oldVersion = $env:HNM_VERSION
$oldArch = $env:PROCESSOR_ARCHITECTURE
$oldWowArch = $env:PROCESSOR_ARCHITEW6432
New-Item -ItemType Directory -Path $testRoot | Out-Null
$env:HNM_INSTALL_DIR = Join-Path $testRoot 'bin with spaces'
$env:HNM_VERSION = "v$env:HNM_TEST_VERSION"
$env:HNM_TEST_MODE = ''
function Invoke-RestMethod { return @{ tag_name = "v$env:HNM_TEST_VERSION" } }
function Invoke-WebRequest {
    param($Uri, $OutFile, [switch]$UseBasicParsing)
    if ($env:HNM_TEST_MODE -eq 'network') { throw 'Simulated network failure' }
    if ($env:HNM_TEST_MODE -eq 'missing' -and $Uri.EndsWith('.zip')) { throw '404 asset missing' }
    if ($Uri.EndsWith('.sha256')) {
        $hash = (Get-FileHash $fixture -Algorithm SHA256).Hash
        if ($env:HNM_TEST_MODE -eq 'checksum') { $hash = '0' * 64 }
        if ($env:HNM_TEST_MODE -eq 'bad-checksum') { $hash = 'invalid' }
        if ($env:HNM_TEST_MODE -eq 'archive') { $hash = (Get-FileHash $OutFile.Replace('.sha256', '') -Algorithm SHA256).Hash }
        "$hash  archive.zip" | Set-Content -LiteralPath $OutFile -Encoding ascii
    } elseif ($env:HNM_TEST_MODE -eq 'archive') {
        'not a zip' | Set-Content -LiteralPath $OutFile
    } else {
        $expectedVersion = $env:HNM_VERSION
        if ($expectedVersion -eq 'latest') { $expectedVersion = "v$env:HNM_TEST_VERSION" }
        if (!$Uri.EndsWith("hnm-$expectedVersion-x86_64-pc-windows-msvc.zip")) { throw "Wrong URL: $Uri" }
        Copy-Item -LiteralPath $fixture -Destination $OutFile
    }
}
function Expect-Failure {
    $before = (Get-FileHash "$env:HNM_INSTALL_DIR/hnm.exe").Hash
    $failed = $false
    try { & "$root/install.ps1" } catch { $failed = $true }
    if (!$failed) { throw "Expected failure: $env:HNM_TEST_MODE" }
    if ((Get-FileHash "$env:HNM_INSTALL_DIR/hnm.exe").Hash -ne $before) { throw 'Old binary changed on failure' }
}
try {
    Get-Content "$root/install.ps1" -Raw | Invoke-Expression
    & "$root/install.ps1"
    if ((& hnm --version) -ne "hnm $env:HNM_TEST_VERSION") { throw 'Version failed' }
    $env:HNM_VERSION = 'latest'
    & "$root/install.ps1"
    $env:HNM_VERSION = "v$env:HNM_TEST_VERSION"
    & hnm init "$testRoot/project" --stack rust
    if ($LASTEXITCODE) { throw 'Init failed' }
    if (!(Get-Item "$testRoot/project/CLAUDE.md").LinkType) { throw 'Missing link' }
    'user config' | Set-Content "$testRoot/project/AGENTS.md"
    & "$root/install.ps1"
    if ((Get-Content "$testRoot/project/AGENTS.md") -ne 'user config') { throw 'Config changed' }
    & hnm init "$testRoot/dry" --dry-run
    if ($LASTEXITCODE -or (Test-Path "$testRoot/dry")) { throw 'Dry run failed' }
    foreach ($env:HNM_TEST_MODE in @('network', 'missing', 'checksum', 'bad-checksum', 'archive')) { Expect-Failure }
    $env:HNM_TEST_MODE = ''
    $env:HNM_VERSION = 'v99.0.0'
    Expect-Failure
    $env:HNM_VERSION = '../invalid'
    Expect-Failure
    $env:HNM_VERSION = "v$env:HNM_TEST_VERSION"
    $env:PROCESSOR_ARCHITECTURE = 'ARM64'
    $env:PROCESSOR_ARCHITEW6432 = ''
    Expect-Failure
    $env:PROCESSOR_ARCHITECTURE = $oldArch
    $env:PROCESSOR_ARCHITEW6432 = $oldWowArch
    # A locked executable must survive a failed upgrade.
    $lock = [IO.File]::Open("$env:HNM_INSTALL_DIR/hnm.exe", 'Open', 'Read', 'Read')
    try { Expect-Failure } finally { $lock.Dispose() }
    Write-Output 'Windows installer regression tests passed'
} finally {
    $env:PATH = $oldPath
    $env:HNM_INSTALL_DIR = $oldDest
    $env:HNM_VERSION = $oldVersion
    $env:PROCESSOR_ARCHITECTURE = $oldArch
    $env:PROCESSOR_ARCHITEW6432 = $oldWowArch
    Remove-Item -LiteralPath $testRoot -Recurse -Force
}
