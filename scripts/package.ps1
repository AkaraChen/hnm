$ErrorActionPreference = 'Stop'
$version = (Select-String '^version = "([^"]+)"' Cargo.toml).Matches[0].Groups[1].Value
$asset = "hnm-v$version-x86_64-pc-windows-msvc.zip"
New-Item -ItemType Directory -Path dist -Force | Out-Null
Compress-Archive -Path target/x86_64-pc-windows-msvc/release/hnm.exe -DestinationPath "dist/$asset"
$hash = (Get-FileHash "dist/$asset" -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText((Join-Path (Get-Location) "dist/$asset.sha256"), "$hash  $asset`n", [Text.Encoding]::ASCII)
