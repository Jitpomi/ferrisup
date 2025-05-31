# Embedded Template Refactoring Plan

The current embedded template has an issue where microcontroller-specific files are not correctly applied through the transformation system. This document outlines a plan to refactor the template for better organization and proper file transformations.

## Current Issues

1. The template's transformation pattern attempts to match "src/main.rs" but the system may be looking for source files in a different way
2. All the target-specific main.rs files (main.rs.esp32, etc.) are copied to the project root instead of replacing src/main.rs
3. The template structure is flat, which makes it harder to organize files by microcontroller

## Proposed Structure

Instead of using file transformations, we can create a more explicit structure with conditionally included files:

```
templates/embedded/
├── template.json
├── common/
│   ├── .cargo/
│   │   └── config.toml
│   └── memory.x
├── src/
│   └── main.rs  (generic version)
├── mcu/
│   ├── rp2040/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml.template
│   ├── esp32/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml.template
│   ├── stm32/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml.template
│   └── arduino/
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml.template
└── README.md
```

## Template Configuration Update

The template.json would use conditional file inclusion rather than transformations:

```json
{
  "files": [
    {
      "source": "common/.cargo/config.toml",
      "target": ".cargo/config.toml"
    },
    {
      "source": "common/memory.x",
      "target": "memory.x"
    },
    {
      "source": "README.md",
      "target": "README.md"
    }
  ],
  "conditional_files": [
    {
      "when": "mcu_target == 'rp2040'",
      "files": [
        {
          "source": "mcu/rp2040/src/main.rs",
          "target": "src/main.rs"
        },
        {
          "source": "mcu/rp2040/Cargo.toml.template",
          "target": "Cargo.toml"
        }
      ]
    },
    {
      "when": "mcu_target == 'esp32'",
      "files": [
        {
          "source": "mcu/esp32/src/main.rs",
          "target": "src/main.rs"
        },
        {
          "source": "mcu/esp32/Cargo.toml.template",
          "target": "Cargo.toml"
        }
      ]
    }
  ]
}
```

## Implementation Steps

1. Create the new directory structure
2. Move existing files to their appropriate locations
3. Update the template.json configuration
4. Enhance the template system to support conditional file inclusion
5. Test with different microcontroller targets

## Benefits

1. More explicit control over which files are included for each target
2. Better organization of template files
3. Easier to maintain and update
4. More intuitive for template authors to understand
5. Reduces reliance on transformation logic which may be error-prone
