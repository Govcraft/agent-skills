# agent-skills-rs

Rust implementation of the [Agent Skills](https://agentskills.io) open standard for parsing, validating, and working with AI agent skill packages.

## What are Agent Skills?

Agent Skills are a portable format for packaging instructions that AI agents can discover and execute. Each skill is a directory containing a `SKILL.md` file with YAML frontmatter (metadata) and markdown body (instructions). See the [specification](https://agentskills.io) for complete format details.

## Crates

| Crate | Use Case | Install |
|-------|----------|---------|
| [`agent-skills`](./crates/agent-skills) | Embed parsing and validation in your Rust application | `cargo add agent-skills` |
| [`agent-skills-cli`](./crates/agent-skills-cli) | Validate skills from the command line or CI | [Multiple options](#quick-start) |

## Quick Start

Install the CLI (no Rust required):

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Govcraft/agent-skills/releases/download/v0.4.2/agent-skills-cli-installer.sh | sh
```

Other installation methods: [npm](https://www.npmjs.com/package/@govcraft/agent-skills), [Cargo](https://crates.io/crates/agent-skills-cli), or [PowerShell](./crates/agent-skills-cli/README.md#powershell-windows).

Validate a skill:

```bash
agent-skills validate ./my-skill
```

Or parse skills in your Rust code:

```rust
use agent_skills::Skill;

let skill = Skill::parse(content)?;
println!("Loaded: {}", skill.name());
```

## Documentation

- [Agent Skills Specification](https://agentskills.io)
- [agent-skills API docs](https://docs.rs/agent-skills)
- [agent-skills-cli on crates.io](https://crates.io/crates/agent-skills-cli)

## License

MIT
