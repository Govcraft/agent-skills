# Issue 1: Unicode skill names

Keep the SkillName newtype and SkillNameError public API. Update the pure
validate_skill_name function to count Unicode scalar values and accept Unicode
alphanumeric characters whose lowercase mapping is unchanged, plus existing
hyphen syntax. Preserve input exactly; do not normalize or change directory
matching. Document scalar limits and normalization behavior.

Test accented, uncased, numeric, and non-BMP names; 64/65 scalar boundaries;
Unicode uppercase/titlecase and punctuation rejection; scalar error positions;
and serialized round trips. Remove test lint suppression using Result tests.
Run cargo check, targeted nextest, and clippy. This is a patch-level correctness
fix, with version and release managed by the integration agent.
