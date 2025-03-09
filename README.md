# rust-sql-to-nosql

An exploration/rough-sketch of a lua based BNF grammar for redis, could through effort be modified to cover any nosql grammar.
- https://github.com/allen-munsch/rust-sql-to-nosql/blob/main/redis.sql.bnf

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

1. Parse the incoming SQL statement into an Abstract Syntax Tree (AST) via sqlparser
2. Match the statement against predefined transformation rules
3. Render a corresponding Redis command using a template engine (tera)

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
- Nested operations and clauses are currently spotty at best

## Disclaimer

This library provides a low-effort transformation and does not cover all possible SQL scenarios. Always validate the generated NoSQL commands in your specific use case. 

Honestly, just a toy program.

```
SQL: SELECT * FROM users WHERE key = 'user:1001'
Redis: GET user:1001

SQL: SELECT value FROM config WHERE key = 'app:settings'
Redis: GET app:settings

SQL: INSERT INTO users (key, value) VALUES ('user:1002', 'Jane Smith')
Redis: SET user:1002 Jane Smith

SQL: UPDATE settings SET value = 'new-value' WHERE key = 'site:theme'
Redis: SET site:theme new-value

SQL: DELETE FROM cache WHERE key = 'temp:data'
Redis: DEL temp:data

SQL: SELECT * FROM users__hash WHERE key = 'user:1001'
Redis: HGETALL user:1001

SQL: SELECT name, email FROM users__hash WHERE key = 'user:1001'
Redis: HMGET user:1001 name email

SQL: INSERT INTO users__hash (key, name, email, age) VALUES ('user:1003', 'Alice Jones', 'alice@example.com', '29')
Redis: HSET user:1003 age 29 name Alice Jones email alice@example.com

SQL: UPDATE users__hash SET status = 'active', last_login = '2023-06-15' WHERE key = 'user:1001'
Redis: HSET user:1001 status active last_login 2023-06-15

SQL: DELETE FROM users__hash WHERE key = 'user:temp'
Redis: DEL user:temp

SQL: DELETE FROM users__hash WHERE key = 'user:1001' AND field = 'temporary_token'
Error: Transformation failed: No matching pattern for: DELETE FROM users__hash WHERE key = 'user:1001' AND field = 'temporary_token'

SQL: SELECT * FROM posts__list WHERE key = 'user:1001:posts'
Redis: LRANGE user:1001:posts 0 -1

SQL: SELECT * FROM timeline__list WHERE key = 'global:timeline' LIMIT 10
Redis: LRANGE global:timeline 0 -1

SQL: SELECT * FROM messages__list WHERE key = 'chat:1001' AND index = 0
Redis: LINDEX chat:1001 0

SQL: INSERT INTO logs__list (key, value) VALUES ('app:logs', 'User logged in')
Redis: RPUSH app:logs User logged in

SQL: UPDATE notifications__list SET value = 'Updated message' WHERE key = 'user:1001:notifications' AND index = 2
Error: Transformation failed: No matching pattern for: UPDATE notifications__list SET value = 'Updated message' WHERE key = 'user:1001:notifications' AND index = 2

SQL: DELETE FROM queue__list WHERE key = 'task:queue'
Redis: DEL task:queue

SQL: DELETE FROM posts__list WHERE key = 'user:1001:posts' AND value = 'spam message'
Error: Transformation failed: No matching pattern for: DELETE FROM posts__list WHERE key = 'user:1001:posts' AND value = 'spam message'

SQL: SELECT * FROM followers__set WHERE key = 'user:1001:followers'
Redis: SMEMBERS user:1001:followers

SQL: SELECT * FROM tags__set WHERE key = 'post:1001:tags' AND member = 'important'
Redis: SISMEMBER post:1001:tags important

SQL: INSERT INTO interests__set (key, member) VALUES ('user:1001:interests', 'technology')
Redis: SADD user:1001:interests technology

SQL: DELETE FROM blocked__set WHERE key = 'user:1001:blocked'
Redis: DEL user:1001:blocked

SQL: DELETE FROM tags__set WHERE key = 'post:1001:tags' AND member = 'temporary'
Error: Transformation failed: No matching pattern for: DELETE FROM tags__set WHERE key = 'post:1001:tags' AND member = 'temporary'

SQL: SELECT * FROM leaderboard__zset WHERE key = 'game:global'
Redis: ZRANGEBYSCORE game:global -inf +inf

SQL: SELECT * FROM ranking__zset WHERE key = 'users:points' AND score > 1000
Redis: ZRANGEBYSCORE users:points (1000 +inf

SQL: SELECT * FROM activity__zset WHERE key = 'site:activity' ORDER BY score DESC
Redis: ZRANGEBYSCORE site:activity -inf +inf

SQL: INSERT INTO leaderboard__zset (key, member, score) VALUES ('game:week1', 'user:1001', '2500')
Redis: ZADD game:week1 2500 user:1001

SQL: UPDATE scores__zset SET score = '3000' WHERE key = 'contest:final' AND member = 'user:1002'
Error: Transformation failed: No matching pattern for: UPDATE scores__zset SET score = '3000' WHERE key = 'contest:final' AND member = 'user:1002'

SQL: DELETE FROM leaderboard__zset WHERE key = 'game:old'
Redis: DEL game:old

SQL: DELETE FROM ranking__zset WHERE key = 'users:points' AND member = 'user:deleted'
Error: Transformation failed: No matching pattern for: DELETE FROM ranking__zset WHERE key = 'users:points' AND member = 'user:deleted'
```

## cargo build --release && target/release/sql_redis --list-patterns

```
Supported SQL to Redis patterns:

SELECT Operations:
  1. string_get (matcher: is_string_get)
     SQL: SELECT * FROM table WHERE key = 'value'
     Redis: GET value
  2. string_get (matcher: is_string_get_value)
     SQL: SELECT value FROM table WHERE key = 'value'
     Redis: GET value
  3. hash_getall (matcher: is_hash_getall)
     SQL: SELECT * FROM table__hash WHERE key = 'value'
     Redis: HGETALL value
  4. hash_get (matcher: is_hash_get)
     SQL: SELECT field FROM table__hash WHERE key = 'value'
     Redis: HGET value field
  5. hash_hmget (matcher: is_hash_hmget)
     SQL: SELECT field1, field2 FROM table__hash WHERE key = 'value'
     Redis: HMGET value field1 field2...
  6. list_getall (matcher: is_list_getall)
     SQL: SELECT * FROM table__list WHERE key = 'value'
     Redis: LRANGE value 0 -1
  7. list_get_index (matcher: is_list_get_index)
     SQL: SELECT * FROM table__list WHERE key = 'value' AND index = n
     Redis: LINDEX value n
  8. list_get_range (matcher: is_list_get_range)
     SQL: SELECT * FROM table__list WHERE key = 'value' LIMIT n
     Redis: LRANGE value 0 n-1
  9. set_getall (matcher: is_set_getall)
     SQL: SELECT * FROM table__set WHERE key = 'value'
     Redis: SMEMBERS value
  11. zset_getall (matcher: is_zset_getall)
     SQL: SELECT * FROM table__zset WHERE key = 'value'
     Redis: ZRANGEBYSCORE value -inf +inf
  12. zset_get_score_range (matcher: is_zset_get_score_range)
     SQL: SELECT * FROM table__zset WHERE key = 'value' AND score > n
     Redis: ZRANGEBYSCORE value (n +inf
  13. zset_get_reversed (matcher: is_zset_get_reversed)
     SQL: SELECT * FROM table__zset WHERE key = 'value' ORDER BY score DESC
     Redis: ZREVRANGEBYSCORE value +inf -inf

INSERT Operations:
  10. set_ismember (matcher: is_set_ismember)
     SQL: SELECT * FROM table__set WHERE key = 'value' AND member = 'member'
     Redis: SISMEMBER value member
  14. string_set (matcher: is_string_set)
     SQL: INSERT INTO table (key, value) VALUES ('key', 'value')
     Redis: SET key value
  15. hash_set (matcher: is_hash_set)
     SQL: INSERT INTO table__hash (key, field1, field2) VALUES ('key', 'value1', 'value2')
     Redis: HSET key field1 value1 field2 value2
  16. list_push (matcher: is_list_push)
     SQL: INSERT INTO table__list (key, value) VALUES ('key', 'value')
     Redis: RPUSH key value
  17. set_add (matcher: is_set_add)
     SQL: INSERT INTO table__set (key, member) VALUES ('key', 'member')
     Redis: SADD key member
  18. zset_add (matcher: is_zset_add)
     SQL: INSERT INTO table__zset (key, member, score) VALUES ('key', 'member', 'score')
     Redis: ZADD key score member
  22. zset_update (matcher: is_zset_update)
     SQL: UPDATE table__zset SET score = 'new-score' WHERE key = 'key' AND member = 'member'
     Redis: ZADD key new-score member
  28. set_delete (matcher: is_set_delete)
     SQL: DELETE FROM table__set WHERE key = 'key'
     Redis: DEL key
  29. set_delete_member (matcher: is_set_delete_member)
     SQL: DELETE FROM table__set WHERE key = 'key' AND member = 'member'
     Redis: SREM key member
  30. zset_delete (matcher: is_zset_delete)
     SQL: DELETE FROM table__zset WHERE key = 'key'
     Redis: DEL key
  31. zset_delete_member (matcher: is_zset_delete_member)
     SQL: DELETE FROM table__zset WHERE key = 'key' AND member = 'member'
     Redis: ZREM key member

UPDATE Operations:
  19. string_update (matcher: is_string_update)
     SQL: UPDATE table SET value = 'new-value' WHERE key = 'key'
     Redis: SET key new-value
  20. hash_update (matcher: is_hash_update)
     SQL: UPDATE table__hash SET field1 = 'value1', field2 = 'value2' WHERE key = 'key'
     Redis: HSET key field1 value1 field2 value2
  21. list_update (matcher: is_list_update)
     SQL: UPDATE table__list SET value = 'new-value' WHERE key = 'key' AND index = 0
     Redis: LSET key 0 new-value

DELETE Operations:
  23. del (matcher: is_string_delete)
     SQL: DELETE FROM table WHERE key = 'key'
     Redis: DEL key
  24. hash_delete (matcher: is_hash_delete)
     SQL: DELETE FROM table__hash WHERE key = 'key'
     Redis: DEL key
  25. hash_delete_field (matcher: is_hash_delete_field)
     SQL: DELETE FROM table__hash WHERE key = 'key' AND field = 'field'
     Redis: HDEL key field
  26. list_delete (matcher: is_list_delete)
     SQL: DELETE FROM table__list WHERE key = 'key'
     Redis: DEL key
  27. list_delete_value (matcher: is_list_delete_value)
     SQL: DELETE FROM table__list WHERE key = 'key' AND value = 'value'
     Redis: LREM key 0 value

```
