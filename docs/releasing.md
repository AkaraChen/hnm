# Releasing hnm

[← Back to README](../README.md)

1. Update `Cargo.toml` and `Cargo.lock` to the next stable version; merge the PR.
2. Push the matching tag, for example `git tag v0.2.1 && git push origin v0.2.1`.
3. The Release workflow tests and packages all five native targets, uploads the
   archives, per-file SHA-256 checksums, and installers to a draft, then publishes
   it only after every build passes. A final matrix installs from the public
   Release and runs `--version` and `init`.

Only the publish job needs `contents: write` through `GITHUB_TOKEN`; builds and
PR checks are read-only. No personal token or external service is required.
Tags must exactly match the Cargo version. Failed draft uploads can be rerun;
published assets are never overwritten (fixes require a new version).
Each Unix archive is named `hnm-vX.Y.Z-<rust-target>.tar.gz`, contains `hnm`, and has a
matching `.sha256` file. `install.sh` and `install.sh.sha256` are also assets.

Windows uses `hnm-vX.Y.Z-x86_64-pc-windows-msvc.zip` containing `hnm.exe`.
`install.ps1` and its SHA-256 file are also Release assets.
The short Unix pipe can hide the initial curl failure exit code; automation needing
strict failure propagation should download successfully before invoking sh.
