# Specification baseline

The [bundled specification](agent-skills-specification.md) records the upstream source, revision, and retrieval date. It contains the upstream MDX verbatim after a provenance comment. Update the snapshot and its provenance together when reviewing a new upstream revision.

The October 2026 review found clarifications rather than a new format: additional files and directories are allowed, metadata maps strings to strings, allowed-tools is a space-separated string, and name examples explicitly include digits. Unicode names and character-based limits were already required before this review.

## Conformance policy

The written [format specification](https://agentskills.io/specification) defines strict conformance. The [reference validator](https://github.com/agentskills/agentskills/tree/69ef37e9424c0a7ea9dd2293b559e43ec8176379/skills-ref) is useful for comparison but is not an exact substitute for the specification. It accepts lowercase skill.md filenames, empty compatibility strings, and allowed-tools sequences in cases where the written format requires SKILL.md, a nonempty compatibility string, and a space-separated tools string.

Unknown top-level fields should produce actionable errors in strict validation so misspellings do not silently discard data. Additional properties belong inside metadata. This strictness matches the reference validator; it is not a newly added format constraint.

The [client integration guide](https://agentskills.io/client-implementation/adding-skills-support#lenient-validation) recommends optionally loading usable skills with diagnostics for cosmetic problems such as directory mismatches or overlong names. Such a policy must be explicit and separate from strict validation. It does not change the format requirements.

## Updating the baseline

1. Fetch upstream and record the exact commit being reviewed.
2. Compare docs/specification.mdx against the pinned revision, separating changed requirements from editorial changes.
3. Review reference-validator changes and client guidance separately.
4. Replace the bundled snapshot, preserving the provenance comment with the new revision and retrieval date.
5. Exercise boundary and interoperability cases before changing validation behavior.
