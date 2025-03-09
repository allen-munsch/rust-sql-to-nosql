# rust-sql-to-nosql

## Overview

This Rust library provides a flexible and powerful tool for transforming SQL queries into NoSQL (specifically Redis) commands. By leveraging a rule-based approach, the library can intelligently convert various SQL statements into equivalent NoSQL operations.

## Key Features

- 🔄 Converts SQL statements to Redis commands
- 🧩 Flexible, extensible rule-based transformation system
- 🚀 High-performance Rust implementation
- 📦 Supports multiple SQL statement types

## Supported Transformations

The library currently supports transformations for:

- `SELECT` statements
- `INSERT` statements
- `UPDATE` statements
- `DELETE` statements

### Example Transformations

| SQL Query | Redis Command | Description |
|-----------|---------------|-------------|
| `SELECT * FROM users WHERE key = 'user123'` | `HGETALL users:user123` | Retrieve all fields for a specific user |
| `INSERT INTO users (key, name, age) VALUES ('user123', 'John', 30)` | `HMSET users:user123 name "John" age 30` | Insert a new user record |
| `UPDATE users SET name = 'Jane' WHERE key = 'user123'` | `HSET users:user123 name "Jane"` | Update a specific user field |
| `DELETE FROM users WHERE key = 'user123'` | `DEL users:user123` | Delete a user record |

## How It Works

The library uses a template-based transformation approach:

1. Parse the incoming SQL statement into an Abstract Syntax Tree (AST)
2. Match the statement against predefined transformation rules
3. Render a corresponding Redis command using a template engine

## Getting Started

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sql-to-redis = { git = "https://github.com/allen-munsch/rust-sql-to-nosql" }
```

### Basic Usage

```rust
use sql_to_redis::SqlToRedisTransformer;

fn main() -> Result<(), SqlRedisError> {
    let transformer = SqlToRedisTransformer::new()?;
    let redis_command = transformer.transform("SELECT * FROM users WHERE key = 'user123'")?;
    println!("{}", redis_command);
    Ok(())
}
```

## Extending the Library

You can easily extend the transformation capabilities by:

- Adding new transformation rules
- Creating custom templates
- Supporting additional SQL statement types

## Performance

Written in Rust, this library offers:
- Zero-cost abstractions
- Efficient parsing and transformation
- Minimal runtime overhead

## Limitations

- Currently focused on Redis as the target NoSQL database
- Supports a subset of SQL operations
- Complex queries might require custom rule implementation

## Disclaimer

This library provides a best-effort transformation and may not cover all possible SQL scenarios. Always validate the generated NoSQL commands in your specific use case.