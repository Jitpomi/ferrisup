# FerrisUp Template Authoring Guide

This guide explains how to create and customize templates for the FerrisUp CLI, including the advanced features like conditional content, variable substitution, and custom next steps.

## Table of Contents

1. [Template Directory Structure](#template-directory-structure)
2. [Template Configuration File](#template-configuration-file)
3. [Variables and Substitution](#variables-and-substitution)
4. [Conditional Logic](#conditional-logic)
5. [File Transformations](#file-transformations)
6. [Best Practices](#best-practices)
7. [Examples](#examples)

## Template Directory Structure

A FerrisUp template consists of:

- A root directory with the template name
- A `template.json` configuration file
- Template files that will be copied to the target project

Example:
```
templates/
└── my-template/
    ├── template.json
    ├── README.md
    ├── src/
    │   └── main.rs
    └── Cargo.toml.template
```

## Template Configuration File

The `template.json` file defines the template's behavior and options. Here's a complete reference:

```json
{
  "name": "template-name",
  "description": "Template description",
  "type": "binary|library",
  "files": [
    {
      "source": "relative/path/in/template",
      "target": "destination/path/in/project"
    }
  ],
  "options": [
    {
      "name": "variable_name",
      "description": "User-friendly description",
      "type": "select|input|boolean",
      "options": ["option1", "option2"],
      "default": "option1"
    }
  ],
  "transformations": [
    {
      "pattern": "path/to/file",
      "replacement": {
        "condition1": "replacement1",
        "condition2": "replacement2"
      }
    }
  ],
  "dependencies": {
    "default": [
      "dependency1 = \"0.1\"",
      "dependency2 = { version = \"0.2\", features = [\"feature1\"] }"
    ],
    "condition1": [
      "extra_dependency = \"0.3\""
    ]
  },
  "dev-dependencies": {
    "default": [
      "dev-dependency1 = \"0.1\""
    ]
  },
  "next_steps": {
    "default": [
      "cd {{project_name}}",
      "cargo build"
    ],
    "conditional": [
      {
        "when": "variable_name == 'option1'",
        "steps": [
          "Additional steps for option1"
        ]
      }
    ]
  },
  "post_setup_info": {
    "conditional": [
      {
        "when": "variable_name == 'option1'",
        "message": "Additional info for option1"
      }
    ]
  }
}
```

## Variables and Substitution

Templates can use variables in two ways:

1. In template files using Handlebars syntax: `{{variable_name}}`
2. In the template.json configuration for conditional logic

### Built-in Variables

- `{{project_name}}`: The name of the project being created

### Custom Variables

Define custom variables in the `options` section of `template.json`. These variables will be:

1. Automatically prompted to the user during project creation
2. Available for substitution in template files
3. Available for conditional logic in `template.json`

## Conditional Logic

### In Template Files

You can use conditional blocks in template files with Handlebars syntax:

```handlebars
{{#if (eq variable_name "value")}}
This content will only appear if variable_name equals "value"
{{else}}
This is the fallback content
{{/if}}
```

### In template.json

Use the `conditional` property in the `next_steps` and `post_setup_info` sections:

```json
"next_steps": {
  "conditional": [
    {
      "when": "variable_name == 'value'",
      "steps": ["These steps will only appear if the condition is met"]
    }
  ]
}
```

## File Transformations

The `transformations` section allows dynamically selecting different source files based on variable values:

```json
"transformations": [
  {
    "pattern": "source/file/path",
    "replacement": {
      "option1": "source/file/for/option1",
      "option2": "source/file/for/option2"
    }
  }
]
```

### Important Note on File Paths

When using transformations, the `pattern` should match the **target** path from the `files` section, not just the filename. For example:

```json
"files": [
  {
    "source": "main.rs",
    "target": "src/main.rs"
  }
],
"transformations": [
  {
    "pattern": "src/main.rs",  // CORRECT: matches the target path
    "replacement": {
      "option1": "main.rs.option1"
    }
  }
]
```

**INCORRECT** transformation pattern that won't work:
```json
"pattern": "main.rs",  // WRONG: doesn't match the target path
```

## Best Practices

1. **Keep templates modular**: Only include what's necessary
2. **Use descriptive variable names**: Make it clear what each option controls
3. **Provide good defaults**: Users should be able to accept defaults and get a working project
4. **Document next steps**: Include clear instructions on how to work with the generated project
5. **Test with and without interaction**: Ensure templates work in both interactive and non-interactive modes
6. **Use consistent formatting**: Follow Rust style guidelines in template code

## Examples

### Basic Template with Options

```json
{
  "name": "web-server",
  "description": "A web server template",
  "type": "binary",
  "files": [
    {
      "source": "src/main.rs",
      "target": "src/main.rs"
    },
    {
      "source": "Cargo.toml.template",
      "target": "Cargo.toml"
    },
    {
      "source": "README.md",
      "target": "README.md"
    }
  ],
  "options": [
    {
      "name": "framework",
      "description": "Web framework to use",
      "type": "select",
      "options": ["axum", "actix", "rocket"],
      "default": "axum"
    }
  ],
  "next_steps": {
    "default": [
      "cd {{project_name}}",
      "cargo run"
    ],
    "conditional": [
      {
        "when": "framework == 'rocket'",
        "steps": [
          "cd {{project_name}}",
          "# Note: Rocket requires nightly Rust",
          "rustup override set nightly",
          "cargo run"
        ]
      }
    ]
  }
}
```

### Complex Template with Multiple Options

See the embedded template for a complete example of a complex template with multiple options and conditional content.
