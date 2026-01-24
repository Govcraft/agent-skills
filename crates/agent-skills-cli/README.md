# agent-skills-cli

Command-line tool for validating and working with [Agent Skills](https://agentskills.io).

## Installation

Install the `agent-skills` binary using any of these methods:

### Shell (macOS/Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Govcraft/agent-skills/releases/download/v0.4.2/agent-skills-cli-installer.sh | sh
```

### PowerShell (Windows)

```powershell
irm https://github.com/Govcraft/agent-skills/releases/download/v0.4.2/agent-skills-cli-installer.ps1 | iex
```

### npm

```bash
npm install -g @govcraft/agent-skills
```

### Cargo

```bash
cargo install agent-skills-cli
```

## Commands

### validate

Check that a skill conforms to the specification:

```bash
agent-skills validate ./my-skill
```
```
✓ my-skill (./my-skill)
```

Accepts a skill directory, a path to `SKILL.md`, or `-` to read paths from stdin.

### list

Discover skills in a directory:

```bash
agent-skills list ./skills
```
```
data-analysis    Analyze datasets and generate reports...
pdf-processing   Extract text from PDFs, fill forms...
```

Options: `--recursive` to search subdirectories, `--check` to validate each skill, `--long` for full details.

### read-properties

Extract skill metadata as JSON, YAML, or TOML:

```bash
agent-skills read-properties ./my-skill --format json
```
```json
{
  "name": "my-skill",
  "description": "Does something useful."
}
```

### to-prompt

Generate XML for embedding skill metadata in agent prompts:

```bash
agent-skills to-prompt ./skill-one ./skill-two
```

## Scripting and CI

- **Exit codes:** 0 on success, non-zero on failure
- **JSON errors:** `--json` for machine-readable error output
- **Stdin support:** Pass `-` to read paths from stdin
- **Quiet mode:** `-q` suppresses informational output

```bash
agent-skills validate ./my-skill || exit 1
```

## Specification

Skills are validated against the [Agent Skills specification](https://agentskills.io).

## License

MIT
