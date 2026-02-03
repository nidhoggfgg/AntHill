# metadata.json Reference

Complete reference for the AntHill plugin metadata.json file.

## Root Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `plugin_id` | string | Yes | Unique identifier for the plugin (kebab-case) |
| `name` | string | Yes | Human-readable plugin name |
| `version` | string | Yes | Semantic version (e.g., "1.0.0") |
| `plugin_type` | string | Yes | Either "python" or "javascript" |
| `description` | string | Yes | Short description of plugin functionality |
| `author` | string | Yes | Plugin author name |
| `entry_point` | string | Yes | Main file path (e.g., "main.py", "index.js") |
| `min_atom_node_version` | string | Yes | Minimum AntHill version required |
| `groups` | array | No | Parameter group definitions |
| `parameters` | array | No | Parameter definitions |
| `metadata` | object | No | Additional plugin metadata |

## Parameter Groups

Groups organize parameters into logical sections:

```json
{
  "groups": [
    {
      "id": "basic",
      "label": "Basic Settings"
    },
    {
      "id": "advanced",
      "label": "Advanced Options"
    }
  ]
}
```

Each group requires:
- `id`: Unique group identifier (used in parameter `group` field)
- `label`: Display name for the group

## Parameter Types

All parameters share common fields:

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Parameter identifier (used in code) |
| `type` | string | Parameter type (see below) |
| `description` | string | Help text for the parameter |
| `label` | string | Display label |
| `default` | varies | Default value (type-specific) |
| `required` | boolean | Whether parameter is required (default: false) |
| `group` | string | Which group this parameter belongs to |

### Type-Specific Fields

#### string
```json
{
  "name": "username",
  "type": "string",
  "default": "guest",
  "placeholder": "Enter username",
  "choices": ["guest", "admin", "user"]
}
```
- `placeholder`: Optional placeholder text
- `choices`: Optional array of allowed values

#### number
```json
{
  "name": "threshold",
  "type": "number",
  "default": 0.5,
  "validation": {
    "min": 0.0,
    "max": 1.0
  }
}
```
- `validation.min`: Minimum value
- `validation.max`: Maximum value

#### integer
```json
{
  "name": "count",
  "type": "integer",
  "default": 10,
  "validation": {
    "min": 1,
    "max": 100
  }
}
```

#### boolean
```json
{
  "name": "enabled",
  "type": "boolean",
  "default": false
}
```

#### json
```json
{
  "name": "config",
  "type": "json",
  "default": {"key": "value"},
  "description": "Arbitrary JSON configuration"
}
```

#### date
```json
{
  "name": "run_date",
  "type": "date",
  "default": "2026-01-01",
  "format": "YYYY-MM-DD"
}
```
- `format`: Date format pattern

#### select
Single choice from predefined options:
```json
{
  "name": "mode",
  "type": "select",
  "default": "fast",
  "choices": [
    {"label": "Fast Mode", "value": "fast"},
    {"label": "Safe Mode", "value": "safe"},
    {"label": "Verbose Mode", "value": "verbose"}
  ]
}
```
- `choices`: Array of `{label, value}` objects

#### multi_select
Multiple choices from predefined options:
```json
{
  "name": "tags",
  "type": "multi_select",
  "default": ["alpha", "beta"],
  "choices": [
    {"label": "Alpha", "value": "alpha"},
    {"label": "Beta", "value": "beta"},
    {"label": "Release", "value": "release"}
  ],
  "placeholder": "Select tags"
}
```

#### file
File path input with extension filtering:
```json
{
  "name": "config_file",
  "type": "file",
  "default": "/tmp/config.json",
  "accept": [".json", ".yaml", ".yml"]
}
```
- `accept`: Array of allowed file extensions

#### directory
Directory path input:
```json
{
  "name": "output_dir",
  "type": "directory",
  "default": "/tmp/output"
}
```

#### textarea
Long-form text input:
```json
{
  "name": "notes",
  "type": "textarea",
  "default": "",
  "placeholder": "Enter notes here...",
  "description": "Additional notes"
}
```

## Metadata Object

Optional additional plugin metadata:
```json
{
  "metadata": {
    "supports_preview": true,
    "category": "automation",
    "icon": "plugin-icon.png",
    "custom_field": "custom_value"
  }
}
```

Common metadata fields:
- `supports_preview`: Whether plugin supports prepare phase (boolean)
- `category`: Plugin category for organization
- `icon`: Icon filename (if included in plugin package)

## Complete Example

```json
{
  "plugin_id": "file-processor",
  "name": "File Processor",
  "version": "1.0.0",
  "plugin_type": "python",
  "description": "Processes files according to configuration",
  "author": "Your Name",
  "entry_point": "main.py",
  "min_atom_node_version": "0.1.0",
  "groups": [
    {"id": "input", "label": "Input Settings"},
    {"id": "output", "label": "Output Settings"}
  ],
  "parameters": [
    {
      "name": "source_file",
      "type": "file",
      "label": "Source File",
      "description": "File to process",
      "accept": [".txt", ".csv", ".json"],
      "group": "input"
    },
    {
      "name": "mode",
      "type": "select",
      "label": "Processing Mode",
      "description": "How to process the file",
      "default": "normal",
      "choices": [
        {"label": "Normal", "value": "normal"},
        {"label": "Fast", "value": "fast"},
        {"label": "Verbose", "value": "verbose"}
      ],
      "group": "input"
    },
    {
      "name": "output_dir",
      "type": "directory",
      "label": "Output Directory",
      "description": "Where to save results",
      "default": "/tmp/processed",
      "group": "output"
    }
  ],
  "metadata": {
    "supports_preview": true,
    "category": "file-processing"
  }
}
```
