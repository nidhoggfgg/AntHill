---
name: anthill-plugin-dev
description: Guide for developing AntHill plugins. Use when users request creating plugins, extensions, or add-ons for AntHill. Covers plugin structure, metadata.json configuration, parameter handling, prepare/apply phases, and Python/JavaScript implementation patterns.
---

# AntHill Plugin Development

## Overview

AntHill plugins extend the platform with custom functionality. This skill guides you through creating plugins that users can install and run.

## Plugin Structure

A plugin is a ZIP archive containing:

```
your-plugin/
├── metadata.json      # Plugin manifest (required)
├── main.py            # Entry point (Python) or index.js (JavaScript)
├── requirements.txt   # Python dependencies (Python plugins only)
└── README.md          # Optional documentation
```

When installed, plugins are extracted to `{install_root}/plugins/{plugin_id}/`.

## Plugin Types

AntHill supports two plugin types:

| Type | entry_point | Dependencies |
|------|-------------|--------------|
| `python` | `main.py` | `requirements.txt` |
| `javascript` | `index.js` | `package.json` |

## Quick Start: Create a Plugin

### 1. Choose Plugin Type

Select Python or JavaScript based on:
- Required libraries/ecosystem
- Team expertise
- Performance requirements

### 2. Write metadata.json

The metadata.json file defines plugin configuration:

```json
{
  "plugin_id": "my-plugin",
  "name": "My Plugin",
  "version": "0.1.0",
  "plugin_type": "python",
  "description": "What this plugin does",
  "author": "Your Name",
  "entry_point": "main.py",
  "min_atom_node_version": "0.1.0",
  "parameters": [
    {
      "name": "text",
      "type": "string",
      "description": "Text to process",
      "label": "Input Text"
    }
  ],
  "metadata": {
    "supports_preview": true
  }
}
```

**Critical fields:**
- `plugin_id`: Unique identifier (kebab-case, no spaces)
- `entry_point`: Relative path to main file
- `min_atom_node_version`: Minimum AntHill version required

### 3. Implement Entry Point

Python plugins receive parameters via environment variables:

```python
import json
import os

raw_params = os.getenv("ANTHILL_PLUGIN_PARAMS")
phase = os.getenv("ANTHILL_PHASE", "apply")

params = json.loads(raw_params) if raw_params else {}
```

JavaScript plugins:

```javascript
const rawParams = process.env.ANTHILL_PLUGIN_PARAMS;
const phase = process.env.ANTHILL_PHASE || 'apply';

const params = rawParams ? JSON.parse(rawParams) : {};
```

### 4. Implement Two-Phase Execution

AntHill plugins support two execution phases:

**Prepare Phase** (`ANTHILL_PHASE=prepare`):
- Generate preview/plan
- Output JSON describing what will happen
- Do NOT make changes

**Apply Phase** (`ANTHILL_PHASE=apply`):
- Execute actual operation
- May receive preview plan via `ANTHILL_PREVIEW_PLAN`

```python
if phase == "prepare":
    plan = {
        "phase": "prepare",
        "targets": ["/tmp/file1.txt", "/tmp/file2.txt"],
        "message": "Will create 2 files"
    }
    print(json.dumps(plan))
elif phase == "apply":
    # Execute operation
    print("Executing...")
```

## Parameter Reference

### Accessing Parameters in Code

Parameters are passed as JSON via `ANTHILL_PLUGIN_PARAMS`:

```python
import json
import os

raw = os.getenv("ANTHILL_PLUGIN_PARAMS")
params = json.loads(raw) if raw else {}

# Access parameter values
text = params.get("text", "default")
count = params.get("count", 1)
```

### Supported Parameter Types

| Type | Description | Example Value |
|------|-------------|---------------|
| `string` | Text input | `"hello"` |
| `number` | Floating-point | `3.14` |
| `integer` | Whole number | `42` |
| `boolean` | True/false | `true` |
| `json` | Arbitrary JSON | `{"key": "value"}` |
| `date` | Date string | `"2026-01-01"` |
| `select` | Single choice | `"option_a"` |
| `multi_select` | Multiple choices | `["a", "b"]` |
| `file` | File path | `"/path/to/file.txt"` |
| `directory` | Directory path | `"/path/to/dir"` |
| `textarea` | Long text | `"Long\nform\ntext"` |

### Parameter Validation

Define validation rules in metadata.json:

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

### Parameter Groups

Organize parameters into groups:

```json
{
  "groups": [
    {"id": "basic", "label": "Basic Settings"},
    {"id": "advanced", "label": "Advanced"}
  ],
  "parameters": [
    {
      "name": "param1",
      "type": "string",
      "group": "basic"
    },
    {
      "name": "param2",
      "type": "boolean",
      "group": "advanced"
    }
  ]
}
```

## Preview System

AntHill plugins support a preview mechanism that allows users to see what will happen before executing the operation. This is critical for plugins that make destructive changes.

### How Preview Works

1. **Prepare Phase** (Preview):
   - User clicks "Preview" or plugin runs with `ANTHILL_PHASE=prepare`
   - Plugin generates a plan describing what will happen
   - Plan is stored with a 10-minute TTL
   - User can review the plan before execution

2. **Apply Phase** (Execution):
   - User clicks "Apply" to approve the preview
   - Plugin runs with `ANTHILL_PHASE=apply`
   - Preview plan is available via `ANTHILL_PREVIEW_PLAN`
   - Plugin executes the actual operation

### Enabling Preview

Set `supports_preview: true` in metadata.json:

```json
{
  "metadata": {
    "supports_preview": true
  }
}
```

If `supports_preview` is false or omitted, the plugin only has an apply phase.

### Preview Plan Structure

The preview plan is a JSON object. Structure is up to you, but typically includes:

```json
{
  "phase": "prepare",
  "message": "Human-readable summary",
  "targets": ["/tmp/file1.txt", "/tmp/file2.txt"],
  "operation": "create",
  "total_size": 2048,
  "estimated_duration": "5s",
  "warnings": ["Existing files will be overwritten"]
}
```

### Complete Preview Example

```python
import json
import os

raw_params = os.getenv("ANTHILL_PLUGIN_PARAMS")
phase = os.getenv("ANTHILL_PHASE", "apply")
preview_plan = os.getenv("ANTHILL_PREVIEW_PLAN")

params = json.loads(raw_params) if raw_params else {}

# === PREPARE PHASE ===
if phase == "prepare":
    # Collect files that will be created
    count = params.get("count", 1)
    output_dir = params.get("output_dir", "/tmp")

    targets = [
        f"{output_dir}/file_{i}.txt"
        for i in range(count)
    ]

    # Generate preview plan
    plan = {
        "phase": "prepare",
        "message": f"Will create {count} file(s) in {output_dir}",
        "targets": targets,
        "operation": "create",
        "file_count": count,
        "warnings": []
    }

    # Add warnings if files already exist
    for target in targets:
        if os.path.exists(target):
            plan["warnings"].append(f"File {target} already exists and will be overwritten")

    print(json.dumps(plan, ensure_ascii=False))
    return

# === APPLY PHASE ===
if phase == "apply":
    # Parse preview plan
    if preview_plan:
        plan = json.loads(preview_plan)
        targets = plan.get("targets", [])
    else:
        # No preview plan, calculate targets directly
        count = params.get("count", 1)
        output_dir = params.get("output_dir", "/tmp")
        targets = [f"{output_dir}/file_{i}.txt" for i in range(count)]

    # Execute the operation
    for target in targets:
        with open(target, 'w') as f:
            f.write("Content from plugin")

    print(json.dumps({
        "status": "success",
        "created_files": targets
    }))
    return
```

### Preview Best Practices

1. **Always validate in prepare phase** - Check for errors before apply
2. **Provide clear summaries** - Users should understand what will happen
3. **Include warnings** - Alert users to potential issues (overwrites, deletions, etc.)
4. **Be accurate** - Preview should match what actually happens in apply
5. **Use preview plan in apply** - Don't recalculate targets, use the plan
6. **Include estimates** - Duration, size, or count help users understand impact

### Preview Without Execution

Some plugins only provide information without making changes:

```python
if phase == "prepare":
    # Gather and display information
    result = {
        "system_info": {...},
        "recommendations": [...]
    }
    print(json.dumps(result))
elif phase == "apply":
    # Usually does nothing for info-only plugins
    print("No changes made - information only")
```

## Plugin Templates

Use templates from `assets/python-plugin-template/` as starting points:

- `metadata.json` - Plugin manifest template
- `main.py` - Python entry point template
- `requirements.txt` - Python dependencies template

## Common Patterns

### File Operations

```python
if phase == "prepare":
    plan = {
        "targets": [file1, file2],
        "operation": "create"
    }
    print(json.dumps(plan))
elif phase == "apply":
    # Create files
    for target in plan.get("targets", []):
        with open(target, 'w') as f:
            f.write(content)
```

### External API Calls

```python
import requests

api_key = params.get("api_key")
response = requests.get(
    "https://api.example.com/data",
    headers={"Authorization": f"Bearer {api_key}"}
)
```

### Error Handling

```python
import sys

try:
    # Plugin logic
    pass
except Exception as e:
    print(json.dumps({"error": str(e)}))
    sys.exit(1)
```

## Best Practices

1. **Always support prepare phase** - Generate previews before making changes
2. **Validate parameters** - Check for required values and valid ranges
3. **Handle errors gracefully** - Return structured error messages
4. **Use parameter groups** - Organize complex parameter lists
5. **Provide defaults** - Make plugins easy to use with sensible defaults
6. **Document parameters** - Clear descriptions help users understand options
7. **Use specific types** - Choose appropriate parameter types for better UI

## Installation

Plugins are installed via:

1. **Upload ZIP file** through API: `POST /api/plugins`
2. **Provide URL** to download plugin package
3. **Specify local file path** for testing

## Advanced Configuration

For detailed metadata.json reference, see [references/metadata-reference.md](references/metadata-reference.md).

Key advanced topics:
- Parameter validation rules
- File type restrictions
- Custom metadata fields
- Version compatibility
