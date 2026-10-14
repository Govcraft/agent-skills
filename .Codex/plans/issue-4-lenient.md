# Issue 4: Opt-in lenient loading

Add LenientSkill and LenientSkillDirectory as distinct representations, with
LoadDiagnostic enum for NameTooLong and NameMismatch. SkillDirectory::load_lenient
will share directory I/O and raw YAML parsing with strict loading, validate all
fields, preserve exact names, and relax only scalar name length and directory
matching. Extract name syntax validation so invalid syntax after character 64
still fails. Keep SkillName and Skill immutable and strict. Add read-only accessors
for all lenient skill fields, path and diagnostics. No CLI behavior changes.

Test strict/lenient divergence, combined warnings, multibyte scalar counts,
required-field and syntax failures, malformed YAML, unknown fields, optional
fields and body preservation. Remove loader test lint suppression and use Result
tests. Check compilation, nextest, clippy, and docs. This additive API warrants a
minor library version bump, managed by the integration agent.
