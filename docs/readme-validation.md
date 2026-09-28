# README validation — KIT-934

Validated on 2026-09-28 against release v0.1.3 and the CLI source on master after
KIT-932 (PR #8, merged). This change updates documentation only.

## Commands and content

- Read `Cargo.toml`, `src/{cli,main,init,plan,stack}.rs`, both installers, bundled
  hook configuration, the specification, PRDs, and ADRs.
- Downloaded the published `install.sh`, ran it with an isolated absolute
  `HNM_INSTALL_DIR`, and verified `hnm --version` and `hnm init --help`.
- In a temporary existing project, `hnm init --dry-run` wrote nothing.
  `hnm init` produced 11 regular files and 2 relative symlinks. A second run
  preserved every file; `--name demo --stack rust --force` updated the generated
  name and commands while leaving an unrelated application file unchanged.
- Both READMEs and installation guides have matching executable code blocks
  (excluding translated comments); all their relative file links resolve.
- `cargo test`: 12 passed. `git diff --check`: passed.
- Clarified an existing documentation discrepancy: default `.` uses the name
  `project`; it does not resolve the current directory name. No CLI code changed.
- Native Windows/macOS execution was not repeated in this Linux environment.
  Their instructions were checked against installer code and release assets.

## First-screen previews

Rendered each complete README with Marked 18.0.14, github-markdown-css 5.9.0 and
Playwright 1.63.0 / Chromium, using system fonts and native horizontally scrollable
code blocks. The article has a maximum width of 1012 px, 32 px desktop padding,
and 16 px mobile padding. These are GitHub-style README-content previews, not
screenshots of GitHub's surrounding repository navigation or file list.

| Viewport | Languages | Themes | Unix block bottom | Windows block bottom |
| --- | --- | --- | --- | --- |
| 1440 × 900 | English, Chinese | Light, dark | 392 px | 540 px |
| 390 × 844 | English, Chinese | Light, dark | 424 px | 596 px |

All eight previews were captured at scroll position zero. Both platform blocks
show installation, `cd`, and `hnm init` within the viewport. Long download URLs
scroll horizontally on mobile, following GitHub code-block behavior; copying the
block retains the full commands. No custom wrapping or reduced font size was used.
Measured code foreground/background contrast is 14.84:1 in light mode and
15.91:1 in dark mode. Native theme colors remain readable; the opaque cover
is below the setup and usage sections. Screenshots are attached to the KIT-934
issue delivery comment.
