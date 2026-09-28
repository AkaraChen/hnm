# PowerShell 5.1+; keep installation inside a block for downloaded execution.
& {
    $ErrorActionPreference = 'Stop'
    $temp = $null
    $stage = $null
    try {
        if ($env:OS -ne 'Windows_NT') { throw 'Supported platform: Windows x64' }
        $arch = $env:PROCESSOR_ARCHITEW6432
        if (!$arch) { $arch = $env:PROCESSOR_ARCHITECTURE }
        if ($arch -ne 'AMD64') { throw 'Supported architecture: Windows x64' }
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        $repo = 'https://github.com/AkaraChen/hnm/releases'
        $version = $env:HNM_VERSION
        if (!$version -or $version -eq 'latest') {
            $release = Invoke-RestMethod 'https://api.github.com/repos/AkaraChen/hnm/releases/latest'
            $version = $release.tag_name
        }
        if ($version -cnotmatch '^v[0-9]+\.[0-9]+\.[0-9]+$') { throw 'Version must be a stable vX.Y.Z tag' }
        $dest = $env:HNM_INSTALL_DIR
        if (!$dest) { $dest = Join-Path $HOME '.local\bin' }
        if ($dest -notmatch '^[A-Za-z]:[\\/]') { throw 'HNM_INSTALL_DIR must be an absolute local drive path' }
        $dest = [IO.Path]::GetFullPath($dest)
        $exe = Join-Path $dest 'hnm.exe'
        $existing = Get-Item -LiteralPath $exe -Force -ErrorAction SilentlyContinue
        if ($existing -and ($existing.PSIsContainer -or ($existing.Attributes -band [IO.FileAttributes]::ReparsePoint))) {
            throw 'Installation target must be a regular file, not a directory or link'
        }
        $temp = Join-Path ([IO.Path]::GetTempPath()) ("hnm-" + [guid]::NewGuid())
        New-Item -ItemType Directory -Path $temp | Out-Null
        $asset = "hnm-$version-x86_64-pc-windows-msvc.zip"
        $archive = Join-Path $temp $asset
        Invoke-WebRequest -UseBasicParsing "$repo/download/$version/$asset" -OutFile $archive
        Invoke-WebRequest -UseBasicParsing "$repo/download/$version/$asset.sha256" -OutFile "$archive.sha256"
        $expected = ((Get-Content -LiteralPath "$archive.sha256" -Raw).Trim() -split '\s+')[0]
        if ($expected -notmatch '^[0-9a-fA-F]{64}$') { throw 'Invalid checksum file' }
        if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne $expected) { throw 'SHA-256 mismatch' }
        Expand-Archive -LiteralPath $archive -DestinationPath (Join-Path $temp 'unpacked')
        $binary = Join-Path $temp 'unpacked\hnm.exe'
        if (!(Test-Path -LiteralPath $binary -PathType Leaf)) { throw 'Archive does not contain hnm.exe' }
        $actual = & $binary --version
        if ($LASTEXITCODE -ne 0 -or $actual -cne "hnm $($version.Substring(1))") { throw 'Binary version check failed' }
        New-Item -ItemType Directory -Path $dest -Force | Out-Null
        $stage = Join-Path $dest ('.hnm-' + [guid]::NewGuid() + '.exe')
        Copy-Item -LiteralPath $binary -Destination $stage
        if ([IO.File]::Exists($exe)) {
            [IO.File]::Replace($stage, $exe, [System.Management.Automation.Language.NullString]::Value)
        } else {
            [IO.File]::Move($stage, $exe)
        }
        if (($env:PATH -split ';') -notcontains $dest) { $env:PATH = "$dest;$env:PATH" }
        Write-Output "Installed $actual to $exe"
        Write-Output "For future terminals, add $dest to your user PATH."
        Write-Output 'hnm init creates symbolic links: enable Windows Developer Mode or use symlink privileges.'
    } finally {
        if ($stage -and (Test-Path -LiteralPath $stage)) { Remove-Item -LiteralPath $stage -Force }
        if ($temp -and (Test-Path -LiteralPath $temp)) { Remove-Item -LiteralPath $temp -Recurse -Force }
    }
}
