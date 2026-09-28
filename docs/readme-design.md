# README design notes

## Direction

A compact editorial cover for a developer CLI: charcoal, ivory paper, and a
vermilion accent. Three connected folders suggest the PRD / ADR / specification
structure. The cover is conceptual artwork, not a screenshot or CLI output.
The project name, value proposition, navigation, and instructions remain readable
text outside the image. The English README retains the existing 质问 terminology.

GitHub-native headings, code blocks, and a single-column layout handle light and
dark themes. The cover has an opaque background and descriptive alt text; it scales
to the content width. A single native GitHub workflow badge and a release link keep the header useful.
Detailed
installation and maintainer instructions live in linked guides.

## References reviewed

Reviewed on 2026-09-28:

- [fnm](https://github.com/Schniz/fnm#readme): a short CLI introduction followed by
  explicit platform installation instructions. Adopted the practical setup order.
- [mise](https://github.com/jdx/mise#readme): direct summary, quick navigation, and a
  numbered quick start. Adopted the path from installation to first useful command.
- [Volta](https://github.com/volta-cli/volta#readme): prominent project identity and
  compact benefit statements. Borrowed the hierarchy, not its wording or artwork.
  This is a visual reference; its current README marks the project unmaintained.

## Artwork provenance

`docs/assets/hnm-cover.jpg` was generated with the user-requested `grok-api image
generate` CLI, model `grok-imagine-image-2.0`. CLI help and authentication were
checked before generation. The prompt requested an editorial illustration of
three ivory folders, orange tabs and fine connecting lines on charcoal, with the
word “hnm”; no interface, terminal output, or other product claims.

The generated image was proportionally resized to 1440 × 617 and encoded as an
optimized JPEG at quality 85. No generated text carries an instruction or feature
claim. The source artwork is intentionally not duplicated in the repository.

The external release badge endpoint returned HTTP 403 during validation, so it
was replaced with a plain release link. The workflow badge is served by GitHub.
