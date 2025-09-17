# YAMP - Yet Another Minimal Parser

[![CI](https://github.com/sanjeevprasad/yamp/actions/workflows/ci.yml/badge.svg)](https://github.com/sanjeevprasad/yamp/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/yamp.svg)](https://crates.io/crates/yamp)
[![Documentation](https://docs.rs/yamp/badge.svg)](https://docs.rs/yamp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight, efficient YAML parser written in Rust that prioritizes comment preservation, simplicity, and avoiding common YAML pitfalls.

## Philosophy

YAMP takes a radically simplified approach to YAML parsing with two core principles:

### 1. Comment Preservation First
Unlike most YAML parsers that discard comments, YAMP treats comments as first-class citizens. Every comment in your YAML file is preserved during parsing and re-emitted exactly where it belongs. This makes YAMP ideal for configuration files that need human-readable documentation.

### 2. Simplicity Through String-Only Values
- **All scalar values are strings** - no implicit type conversion
- **No type confusion** - values like `NO`, `3.10`, `0755` are always strings
- **Predictable behavior** - what you write is what you get

## Features

- ✅ **Full comment preservation** - both inline and standalone comments
- ✅ **Comments survive round-trips** - parse, modify, and emit without losing documentation
- ✅ All scalar values treated as strings (no type guessing)
- ✅ Basic YAML structures (key-value pairs, arrays, nested objects)
- ✅ Indentation-based structure parsing
- ✅ Quoted and unquoted strings
- ✅ Multiline strings (literal `|` and folded `>`)
- ✅ Simple, clean API
- ✅ Zero dependencies
- ✅ Avoids ALL YAML "footguns" by design

## What's Supported

- Simple key-value pairs (values are always strings)
- Nested objects (maps)
- Arrays (sequences)
- Comments (preserved during parsing and emitting)
- Both quoted and unquoted strings
- Multiline strings with literal (`|`) and folded (`>`) styles
- Chomping modes for multiline strings (strip `-`, clip default, keep `+`)

## What's NOT Supported

- Multi-document YAML files (---)
- Anchors and aliases (&, *)
- Tags (!!str, !!int, etc.)
- Flow style collections ({}, [])
- Complex key types
- Merge keys (<<)
- **Any form of implicit typing** - by design!

## Usage

### Comment Preservation

YAMP's killer feature is preserving comments through parse/emit cycles:

```rust
use yamp::{parse, emit};

fn main() {
    let yaml = r#"# Application configuration
name: MyApp        # Application name
version: 1.2.3     # Semantic version

# Server settings
server:
  host: localhost  # Bind address
  port: 8080       # Listen port
"#;

    // Parse the YAML - all comments are preserved
    let parsed = parse(yaml).expect("Failed to parse");

    // Emit it back - comments remain intact!
    let output = emit(&parsed);
    println!("{}", output);
    // Output includes all the original comments
}
```

#### Working with Comments Programmatically

```rust
use yamp::{parse, emit, YamlNode, YamlObject, YamlValue};

fn main() {
    // Create YAML with comments programmatically
    let mut config = YamlObject::new()
        .with("name", "MyApp")
        .with("version", "1.2.3");

    let mut node: YamlNode = config.into();

    // Add a leading comment to the entire document
    node = node.with_leading_comment("Auto-generated configuration file\nDo not edit manually!");

    // Add a trailing comment (stored as inline_comment at root level)
    node = node.with_inline_comment("End of configuration");

    let yaml_string = emit(&node);
    println!("{}", yaml_string);
    /* Output:
    # Auto-generated configuration file
    # Do not edit manually!
    name: MyApp
    version: 1.2.3
    # End of configuration
    */
}
```

#### Modifying YAML While Preserving Comments

```rust
use yamp::{parse, emit, YamlValue};

fn main() {
    let yaml = r#"# Database configuration
# Production settings
database:
  # Primary database host
  host: db.example.com
  # Database name
  name: production_db
  # Connection pool size
  pool_size: 10

# API Settings
api:
  # API endpoint
  endpoint: https://api.example.com
  # Timeout in seconds
  timeout: 30
"#;

    let mut parsed = parse(yaml).expect("Failed to parse");

    // Modify values while preserving all comments
    if let YamlValue::Object(ref mut root) = parsed.value {
        if let Some(db_node) = root.get_mut("database") {
            if let YamlValue::Object(ref mut db) = db_node.value {
                // Update the host - comment is preserved!
                if let Some(host) = db.get_mut("host") {
                    host.value = YamlValue::String("new-db.example.com".to_string());
                }
                // Update pool size - comment is preserved!
                if let Some(pool) = db.get_mut("pool_size") {
                    pool.value = YamlValue::String("20".to_string());
                }
            }
        }
    }

    let output = emit(&parsed);
    println!("{}", output);
    // All original comments are preserved, only values are updated
}
```

#### Complex Comment Scenarios

```rust
use yamp::{parse, emit};

fn main() {
    let yaml = r#"# Application metadata

# General settings
general:
  # Application name and version
  name: MyApp  # Internal name
  version: 1.0.0  # Semantic versioning

  # Feature flags
  features:
    # Enable experimental features
    - experimental  # Use with caution
    # Enable debug mode
    - debug  # Only in development
    # Enable metrics collection
    - metrics

# Database configuration
database:
  # Connection settings
  connections:
    # Primary database
    - host: primary.db
      port: 5432  # PostgreSQL default
    # Read replica
    - host: replica.db
      port: 5432  # Same port as primary

# End of configuration file
# Generated on 2024-01-01"#;

    let parsed = parse(yaml).expect("Failed to parse");
    let output = emit(&parsed);

    // All comments are preserved:
    // - Leading comments before sections
    // - Inline comments after values
    // - Comments within arrays
    // - Trailing comments at the end
    println!("{}", output);
}
```

### String-Only Values

All values are treated as strings, avoiding YAML's type confusion:

```rust
use yamp::{parse, emit, YamlValue};

fn main() {
    let yaml = r#"
name: John Doe
age: 30
active: true
version: 3.10
country: NO
permissions: 0755
"#;

    // Parse YAML
    let parsed = parse(yaml).expect("Failed to parse YAML");

    // Easy access with helper methods
    assert_eq!(parsed.get("age").and_then(|n| n.as_str()), Some("30"));
    assert_eq!(parsed.get("active").and_then(|n| n.as_str()), Some("true"));
    assert_eq!(parsed.get("version").and_then(|n| n.as_str()), Some("3.10")); // Not 3.1!
    assert_eq!(parsed.get("country").and_then(|n| n.as_str()), Some("NO")); // Not false!
    assert_eq!(parsed.get("permissions").and_then(|n| n.as_str()), Some("0755")); // Not 493!

    // Or using traditional approach for more control
    if let YamlValue::Object(map) = &parsed.value {
        let age = &map.get("age").unwrap().value;
        assert_eq!(age, &YamlValue::String("30".to_string()));
    }

    // Emit back to YAML
    let output = emit(&parsed);
    println!("{}", output);
}
```

### Multiline Strings

YAMP supports YAML multiline strings while maintaining the all-strings philosophy:

```rust
use yamp::{parse, YamlValue};

fn main() {
    let yaml = r#"
description: |
  This is a multiline string
  that preserves line breaks.

summary: >
  This is a folded multiline string
  that joins lines with spaces.
"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Object(map) = &parsed.value {
        // Literal style (|) preserves line breaks
        let desc = &map.get("description").unwrap().value;
        if let YamlValue::String(s) = desc {
            assert!(s.contains("\n"));
        }

        // Folded style (>) joins lines with spaces
        let summary = &map.get("summary").unwrap().value;
        if let YamlValue::String(s) = summary {
            assert!(!s.contains("\n"));
            assert!(s.contains("string that joins"));
        }
    }
}
```

## Why No Type System?

YAML's implicit typing leads to countless surprising behaviors and security issues:

### The Problems We Avoid

1. **The Norway Problem**: Country code `NO` becomes `false`
2. **Version Numbers**: `3.10` becomes `3.1`
3. **Octal Confusion**: `0755` becomes `493`
4. **Scientific Notation**: `2e5a8d9` partially parsed as `200000`
5. **Boolean Chaos**: `yes`, `on`, `y`, `true`, `True`, `TRUE` all mean true
6. **Null Confusion**: `~` in paths like `~/.ssh/config` becomes null
7. **Time Values**: `12:34:56` becomes `45296` (base-60)
8. **Special Floats**: `.inf`, `.nan` cause portability issues

### The YAMP Solution

**Everything is a string.** Period.

If you need typed data, parse the strings in your application where you have full control over the conversion rules. This approach is:
- **Predictable**: No surprises based on value format
- **Secure**: No injection attacks through type confusion
- **Portable**: Same behavior across all systems
- **Simple**: One rule to remember

## Comment Handling Pitfalls

While YAMP preserves comments better than most YAML parsers, there are some limitations to be aware of:

### 1. Trailing Comments at Document Root
Trailing comments at the end of the document are stored in the root node's `inline_comment` field rather than a separate `trailing_comment` field. This means:
- At the root level, `inline_comment` serves dual purpose
- When emitting, root-level inline comments appear at the end of the document
- This is a design choice to keep the API simple

### 2. Comment Association
Comments are associated with the nearest following key or value:
```yaml
# This comment belongs to 'name'
name: John

# This comment belongs to 'age'
age: 30
```

Ambiguous cases:
```yaml
name: John
# Is this a trailing comment for 'name' or leading comment for 'age'?
age: 30
```
YAMP associates such comments with the following key ('age' in this example).

### 3. Multi-line Comment Blocks
Consecutive comment lines are combined with newlines:
```yaml
# First line
# Second line
# Third line
key: value
```
The leading comment for 'key' will be: `"First line\nSecond line\nThird line"`

### 4. Inline Comments on Complex Values
Inline comments on array or object values are preserved but may not round-trip perfectly in all cases:
```yaml
server: # This inline comment is preserved
  host: localhost
  port: 8080
```

### 5. Comments in Arrays
Comments within arrays are associated with individual array items:
```yaml
fruits:
  # Comment for apple
  - apple
  # Comment for banana
  - banana
```

## Design Decisions

- **Comment preservation**: Comments are treated as first-class citizens, not discarded metadata
- **No implicit typing**: All scalar values are strings to avoid confusion and bugs
- **Simplicity over completeness**: Only the most common YAML features are supported
- **Round-trip fidelity**: Parse and emit cycles preserve structure and comments (with the caveats above)
- **Performance**: Efficient parsing with minimal allocations
- **Error messages**: Clear, helpful error messages for debugging
- **No external dependencies**: Pure Rust implementation
- **Predictable behavior**: No surprises with type conversion

## When to Use YAMP

YAMP is perfect when you:
- **Need to preserve comments in configuration files**
- **Want to programmatically update YAML while keeping human documentation**
- Want predictable, safe YAML parsing
- Need to preserve exact values (version numbers, permissions, etc.)
- Are building configuration management tools
- Are building security-sensitive applications
- Want to avoid YAML's numerous footguns
- Prefer explicit type conversion in your application

## When NOT to Use YAMP

Don't use YAMP if you:
- Need full YAML 1.2 specification compliance
- Require automatic type inference
- Need anchors/aliases for data deduplication
- Must parse existing YAML files that rely on implicit typing
- Need multi-document YAML support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
yamp = "0.1.1"
```

## Contributing

Contributions are welcome! However, please note that we will NOT accept PRs that:
- Add any form of implicit typing or type inference
- Compromise the simplicity or predictability of the parser
- Add complex YAML features that increase the attack surface

## License

MIT

## Acknowledgments

This parser is inspired by the [StrictYAML](https://github.com/crdoconnor/strictyaml) project and the numerous articles documenting YAML's problematic "features".

YAMP was created to solve a specific problem: the need for a YAML parser that preserves comments while avoiding YAML's type system complexities. Most YAML parsers treat comments as throwaway metadata, but in configuration files, comments are often as important as the data itself.