# cli-wizard-generator

Generate interactive wizards that produce scripts, configs, or other wizards.

## Features

### Wizard Definition

Declarative wizard format (YAML/TOML):

```yaml
name: "Docker Setup Wizard"
description: "Configure a Docker development environment"

questions:
  - id: project_name
    type: text
    prompt: "Project name?"
    validation:
      pattern: "^[a-z][a-z0-9-]*$"
      message: "Must start with letter, only lowercase and hyphens"

  - id: framework
    type: select
    prompt: "Which framework?"
    options:
      - label: "Node.js"
        value: "node"
      - label: "Python"
        value: "python"
      - label: "Rust"
        value: "rust"

  - id: with_database
    type: confirm
    prompt: "Include database?"
    default: true

  - id: database_type
    type: select
    prompt: "Which database?"
    when: "with_database"
    options:
      - label: "PostgreSQL"
        value: "postgres"
      - label: "MySQL"
        value: "mysql"
      - label: "MongoDB"
        value: "mongo"

  - id: ports
    type: multi_select
    prompt: "Expose which ports?"
    options:
      - label: "HTTP (80)"
        value: "80"
      - label: "HTTPS (443)"
        value: "443"
      - label: "Custom..."
        value: "custom"

  - id: custom_port
    type: number
    prompt: "Custom port number?"
    when: "ports contains 'custom'"
    validation:
      min: 1
      max: 65535

output:
  type: file
  template: docker-compose.yml.hbs
  path: "{{project_name}}/docker-compose.yml"
```

### Question Types

**Text:**
- Single line input
- Validation (regex, length)
- Default value

**Password:**
- Hidden input
- Confirmation option

**Number:**
- Integer or float
- Min/max validation

**Select:**
- Single choice
- Scrollable list

**Multi-Select:**
- Multiple choices
- Space to toggle

**Confirm:**
- Yes/No
- Default value

**Path:**
- File/directory path
- Tab completion
- Existence check

### Conditional Logic

Skip questions based on previous answers:

```yaml
- id: advanced_config
  type: confirm
  prompt: "Configure advanced options?"

- id: cache_size
  type: number
  when: "advanced_config == true"
  prompt: "Cache size (MB)?"
```

### Output Types

**Bash Script:**
```yaml
output:
  type: script
  shell: bash
  template: setup.sh.hbs
```

**Multi-Shell:**
```yaml
output:
  type: script
  shells: [bash, zsh, fish, pwsh]
  templates:
    bash: setup.bash.hbs
    zsh: setup.zsh.hbs
    fish: setup.fish.hbs
    pwsh: setup.ps1.hbs
```

**Config Files:**
```yaml
outputs:
  - type: file
    template: config.toml.hbs
    path: config.toml
  - type: file
    template: docker-compose.yml.hbs
    path: docker-compose.yml
```

**Another Wizard:**
```yaml
output:
  type: wizard
  template: generated-wizard.yaml.hbs
```

### Template Engine

Using Handlebars:

```handlebars
# docker-compose.yml
version: "3.8"
services:
  {{project_name}}:
    image: {{framework}}:latest
    ports:
      {{#each ports}}
      - "{{this}}:{{this}}"
      {{/each}}
    {{#if with_database}}
    depends_on:
      - db

  db:
    image: {{database_type}}:latest
    {{/if}}
```

### Preview Mode

Show generated output before writing:
- Syntax highlighting
- Diff against existing
- Confirm or cancel

### Validation

- Real-time validation
- Error messages
- Retry on invalid

## Running Wizards

**Built-in Runner:**
```bash
cli-wizard-generator run my-wizard.yaml
```

**Generate Standalone:**
```bash
cli-wizard-generator compile my-wizard.yaml -o my-wizard-app
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Auto-complete (paths) |
| `enter` | Submit answer |
| `up/down` | Navigate options |
| `space` | Toggle (multi-select) |
| `Ctrl+c` | Cancel wizard |
| `Ctrl+p` | Preview output |
| `Ctrl+z` | Undo last answer |

## Configuration

```toml
# ~/.config/cli-wizard-generator/config.toml
[editor]
syntax_theme = "monokai"

[templates]
path = "~/.config/cli-wizard-generator/templates"
helpers_path = "~/.config/cli-wizard-generator/helpers"

[output]
preview_before_write = true
backup_existing = true
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
serde_yaml = "0.9"
toml = { workspace = true }
handlebars = "6"
regex = "1"
walkdir = "2"
syntect = "5"
```
