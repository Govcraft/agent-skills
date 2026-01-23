# agent-skills

Parse, validate, and work with [Agent Skills](https://agentskills.io)—the open standard for AI agent instruction packages.

## Installation

```bash
cargo add agent-skills
```

## Usage

Parse skill content directly or load a complete skill directory.

### Parse SKILL.md Content

```rust
use agent_skills::Skill;

let content = r#"---
name: my-skill
description: Does something useful.
---
# Instructions
Follow these steps...
"#;

let skill = Skill::parse(content)?;
assert_eq!(skill.name().as_str(), "my-skill");
```

### Load a Skill Directory

```rust
use agent_skills::SkillDirectory;
use std::path::Path;

let dir = SkillDirectory::load(Path::new("./my-skill"))?;
let skill = dir.skill();

println!("Name: {}", skill.name());
println!("Description: {}", skill.description());
```

Validation occurs automatically during parsing. Invalid skills return descriptive errors.

## Key Types

| Type | Purpose |
|------|---------|
| [`Skill`](https://docs.rs/agent-skills/latest/agent_skills/struct.Skill.html) | A parsed skill with frontmatter and body |
| [`SkillDirectory`](https://docs.rs/agent-skills/latest/agent_skills/struct.SkillDirectory.html) | A loaded skill directory with file access |
| [`Frontmatter`](https://docs.rs/agent-skills/latest/agent_skills/struct.Frontmatter.html) | YAML metadata (name, description, license, etc.) |
| [`SkillName`](https://docs.rs/agent-skills/latest/agent_skills/struct.SkillName.html) | A validated skill name |
| [`SkillDescription`](https://docs.rs/agent-skills/latest/agent_skills/struct.SkillDescription.html) | A validated description |

## Specification

This crate implements the [Agent Skills specification](https://agentskills.io).

## License

MIT
