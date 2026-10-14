# Changelog

## 0.5.0 (CLI) / 0.3.0 (library) - 2026-10-14

## Agent Skills conformance and interoperability

This release fixes validation of international skill names and multibyte descriptions, adds explicit lenient loading for embedding clients, and pins the upstream specification used for conformance review.

### Changes

- Accept Unicode alphanumeric skill names while retaining lowercase and hyphen constraints (#1).
- Count Unicode scalar values for name, description, and compatibility limits (#2).
- Reject unknown top-level frontmatter fields with actionable diagnostics. Custom properties remain supported inside metadata (#3).
- Add opt-in lenient loading with structured diagnostics for overlong names and directory mismatches. Strict types and CLI validation retain their guarantees (#4).
- Track the specification snapshot and document its source revision and conformance policy (#5).
- Preserve complete frontmatter values and body text in CRLF files (#6).
- Avoid UTF-8 truncation panics and align Unicode names in CLI list output (#7).
- Fix existing lint and documentation-test failures without lint suppressions.

### Compatibility

The library is version 0.3.0 and the CLI is version 0.5.0. Unknown top-level fields that were previously ignored now fail strict validation. Move custom fields into metadata. Names retain their original spelling without Unicode normalization; strict directory matching remains exact. Lenient loading is an explicit library API and does not relax the validate command.
