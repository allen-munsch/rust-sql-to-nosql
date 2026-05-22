# sql-to-redis

A declarative SQL to Redis command transformer based on pattern matching and BNF grammar.

[![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Overview

`sql-to-redis` transforms ANSI SQL queries into Redis commands. It maps SQL's relational model onto Redis data structures using a naming convention:
- `table` → String (GET/SET)
- `table__hash` → Hash (HGETALL/HSET/HDEL)
- `table__list` → List (LRANGE/RPUSH/LSET/LREM)
- `table__set` → Set (SMEMBERS/SADD/SREM)
- `table__zset` → Sorted Set (ZRANGEBYSCORE/ZADD/ZREM)

For complex operations like aggregates, the library generates Lua scripts executed via `EVAL` within Redis.

## Key Features

- **53 BNF-defined patterns** covering SELECT, INSERT, UPDATE, DELETE across all 5 Redis data types
- **Lua scripting** for aggregates: AVG, SUM, MIN, MAX, STDDEV_POP on zsets, lists, and hashes
- **Pattern matching** with recursive AND expression handling for compound WHERE clauses
- **Score range support**: `>`, `>=`, `<`, `<=`, `BETWEEN` on sorted sets
- **IN operator**: `WHERE key IN (...)` → `MGET`, `WHERE member IN (...)` → `SREM`
- **COUNT aggregation**: native Redis commands (SCARD, ZCARD, ZCOUNT, HLEN, LLEN)
- **Multi-row INSERT**: `VALUES (k,m1), (k,m2)` → `SADD k m1 m2`
- **Rule ordering**: specific patterns tried before general ones to avoid shadowing
- **Deterministic output**: field order preserved from SQL input

## Table Type Convention

| Redis Type | SQL Table Suffix | Example Key | Commands |
|-----------|-----------------|-------------|----------|
| String | `table` (no suffix) | `key = 'user:1001'` | GET, SET, MGET, DEL |
| Hash | `table__hash` | `key = 'user:1001'` | HGETALL, HGET, HMGET, HSET, HDEL, HLEN |
| List | `table__list` | `key = 'user:1001:posts'` | LRANGE, LINDEX, RPUSH, LSET, LREM, LLEN |
| Set | `table__set` | `key = 'user:1001:followers'` | SMEMBERS, SISMEMBER, SADD, SREM, SCARD |
| Sorted Set | `table__zset` | `key = 'game:global'` | ZRANGEBYSCORE, ZADD, ZREM, ZCARD, ZCOUNT |

## Quick Examples

### CRUD Operations

```sql
-- String operations
SELECT * FROM users WHERE key = 'user:1001'                 -- GET user:1001
SELECT value FROM config WHERE key = 'app:settings'         -- GET app:settings
INSERT INTO users (key, value) VALUES ('user:1002', 'Jane') -- SET user:1002 Jane
UPDATE settings SET value = 'new' WHERE key = 'site:theme'  -- SET site:theme new
DELETE FROM cache WHERE key = 'temp:data'                   -- DEL temp:data

-- Hash operations
SELECT * FROM users__hash WHERE key = 'user:1001'           -- HGETALL user:1001
SELECT name, email FROM users__hash WHERE key = 'user:1001' -- HMGET user:1001 name email
INSERT INTO users__hash (key, name, age) VALUES ('u:1', 'Alice', '29')
    → HSET u:1 name Alice age 29
UPDATE users__hash SET status = 'active' WHERE key = 'u:1'  -- HSET u:1 status active
DELETE FROM users__hash WHERE key = 'u:1' AND field = 'tmp' -- HDEL u:1 tmp

-- List operations
SELECT * FROM posts__list WHERE key = 'u:1:posts'           -- LRANGE u:1:posts 0 -1
SELECT * FROM posts__list WHERE key = 'u:1:posts' LIMIT 10  -- LRANGE u:1:posts 0 9
SELECT * FROM posts__list WHERE key = 'u:1:posts' AND index = 0  -- LINDEX u:1:posts 0
SELECT * FROM posts__list WHERE key = 'u:1:posts' AND index < 5  -- LRANGE u:1:posts 0 4
INSERT INTO logs__list (key, value) VALUES ('app:logs', 'msg')    -- RPUSH app:logs msg
UPDATE list__list SET value = 'new' WHERE key = 'k' AND index = 0 -- LSET k 0 new
DELETE FROM posts__list WHERE key = 'k' AND value = 'spam' -- LREM k 0 spam

-- Set operations
SELECT * FROM followers__set WHERE key = 'u:1:followers'    -- SMEMBERS u:1:followers
SELECT * FROM tags__set WHERE key = 'post:1' AND member = 'x' -- SISMEMBER post:1 x
INSERT INTO interests__set (key, member) VALUES ('u:1:int', 'tech') -- SADD u:1:int tech
DELETE FROM tags__set WHERE key = 'post:1' AND member = 'x' -- SREM post:1 x
DELETE FROM tags__set WHERE key = 'post:1' AND member IN ('a','b')  -- SREM post:1 a b

-- Sorted Set operations
SELECT * FROM leaderboard__zset WHERE key = 'game:global'   -- ZRANGEBYSCORE game:global -inf +inf
SELECT * FROM zset__zset WHERE key = 'k' AND score > 1000   -- ZRANGEBYSCORE k (1000 +inf
SELECT * FROM zset__zset WHERE key = 'k' AND score BETWEEN 100 AND 200 -- ZRANGEBYSCORE k 100 200
SELECT * FROM zset__zset WHERE key = 'k' ORDER BY score DESC -- ZREVRANGEBYSCORE k +inf -inf
INSERT INTO zset__zset (key, member, score) VALUES ('game', 'u:1', '2500') -- ZADD game 2500 u:1
UPDATE zset__zset SET score = '3000' WHERE key = 'k' AND member = 'u:1'   -- ZADD k 3000 u:1
DELETE FROM zset__zset WHERE key = 'k' AND member = 'u:1'  -- ZREM k u:1
```

### Multi-Key Queries

```sql
SELECT * FROM users WHERE key IN ('user:1001', 'user:1002')  -- MGET user:1001 user:1002
```

### COUNT Aggregations

```sql
SELECT COUNT(*) FROM users__hash WHERE key = 'user:1001'     -- HLEN user:1001
SELECT COUNT(*) FROM posts__list WHERE key = 'u:1:posts'     -- LLEN u:1:posts
SELECT COUNT(*) FROM followers__set WHERE key = 'u:1:followers' -- SCARD u:1:followers
SELECT COUNT(*) FROM leaderboard__zset WHERE key = 'game'    -- ZCARD game
SELECT COUNT(*) FROM zset__zset WHERE key = 'game'
    AND score BETWEEN 1000 AND 2000                          -- ZCOUNT game 1000 2000
```

### Aggregate Functions (Lua EVAL)

```sql
-- ZSet aggregates (iterates WITHSCORES in Lua)
SELECT AVG(score) FROM leaderboard__zset WHERE key = 'game'  -- EVAL zset_avg 1 game -inf +inf
SELECT SUM(score) FROM zset__zset WHERE key = 'k' AND score > 1000
SELECT MIN(score) FROM zset__zset WHERE key = 'k'
SELECT MAX(score) FROM zset__zset WHERE key = 'k'
SELECT STDDEV_POP(score) FROM zset__zset WHERE key = 'k'

-- Hash field aggregates
SELECT AVG(age) FROM users__hash WHERE key = 'user:1001'     -- EVAL hash_avg 1 user:1001 age
SELECT SUM(salary) FROM dept__hash WHERE key = 'finance'     -- EVAL hash_sum 1 finance salary

-- List value aggregates
SELECT AVG(value) FROM scores__list WHERE key = 'game:scores' -- EVAL list_avg 1 game:scores
SELECT SUM(value) FROM metrics__list WHERE key = 'api:latency'
SELECT MIN(value) FROM temps__list WHERE key = 'sensor:42'
SELECT MAX(value) FROM temps__list WHERE key = 'sensor:42'
```

## Architecture

```text
SQL String
    │
    ▼
┌─────────────┐     ┌──────────────────┐
│  sqlparser  │────▶│      AST         │
│  (parse)    │     │ (Statement enum) │
└─────────────┘     └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │  Rule 1  │  │  Rule 2  │  │  Rule N  │
        │ matches? │  │ matches? │  │ matches? │
        └────┬─────┘  └────┬─────┘  └────┬─────┘
             │   first match wins        │
             └──────────────┬────────────┘
                            ▼
              ┌─────────────────────────┐
              │  Context Builder        │
              │  (extracts key, fields, │
              │   scores, ranges, etc.) │
              └────────────┬────────────┘
                           │
              ┌────────────┼────────────┐
              ▼                         ▼
    ┌──────────────────┐    ┌──────────────────────┐
    │  Tera Template   │    │  Direct Command       │
    │  (plain Redis)   │    │  (Lua EVAL scripts)   │
    │  GET {{ key }}   │    │  EVAL 'lua...' 1 k v  │
    └────────┬─────────┘    └──────────┬───────────┘
             │                         │
             └──────────┬──────────────┘
                        ▼
              ┌──────────────────┐
              │  Redis Command   │
              │  (output string) │
              └──────────────────┘
```

### Module Layout

```
src/
├── lib.rs              # SqlToRedisTransformer entry point
├── main.rs             # CLI with --query, --file, --list-patterns
├── ast/                # SQL AST extraction (select, insert, update, delete)
│   ├── select.rs       # sel_get_key_value, sel_get_score_range, sel_is_count_star, etc.
│   ├── insert.rs       # ins_get_values_as_maps, ins_get_column_value
│   ├── update.rs       # upd_get_assignments, upd_get_key_value
│   └── delete.rs       # get_key_value, get_field_filter, get_member_in_values
├── rules/              # Rule definitions connecting matchers → context builders
│   ├── select.rs       # 34 SELECT rules (aggregates first, then specific→general)
│   ├── insert.rs       # 5 INSERT rules
│   ├── update.rs       # 4 UPDATE rules
│   └── delete.rs       # 10 DELETE rules (specific→general ordering)
├── context/            # Context builders (extract template variables from AST)
│   ├── select.rs       # StringGet, HashGetAll, ListGetRange, ZSetScoreRange, etc.
│   ├── insert.rs       # StringSet, HashSet, SetAdd (multi-row), ZSetAdd
│   ├── update.rs       # StringUpdate, HashUpdate, ListUpdate, ZSetUpdate
│   └── delete.rs       # StringDelete, HashDeleteField, SetDeleteMultiMember
├── templates/          # Tera templates for plain Redis commands
│   └── mod.rs          # 30+ raw templates registered with Tera
├── lua/                # Lua scripting for aggregates and complex operations
│   └── mod.rs          # EvalCommand builder, zset/list/hash aggregate scripts
├── pattern/            # Declarative pattern matching infrastructure
│   ├── combinators.rs  # Pattern trait, Map, Or, AndThen, Pair combinators
│   ├── matchers/       # BNF rule matchers (bool predicates on AST)
│   │   ├── select.rs   # is_string_get, is_zset_aggregate, has_score_between, etc.
│   │   ├── insert.rs   # is_string_set, is_hash_set, is_set_add, etc.
│   │   ├── update.rs   # is_string_update, is_hash_update, is_list_update, etc.
│   │   ├── delete.rs   # is_set_delete_multi_member, has_member_in, etc.
│   │   └── common.rs   # key_equals, score_range, field_equals pattern combinators
│   ├── extractors/     # Alternative extraction path (fallback, partially redundant)
│   ├── cte.rs          # CTE/WITH clause patterns (infrastructure)
│   ├── join.rs         # JOIN pattern matching (infrastructure)
│   └── subquery.rs     # Subquery extraction (infrastructure)
└── commands.rs         # Legacy fallback command generation
```

## Getting Started

### CLI Usage

```bash
# Single query
cargo run -- --query "SELECT * FROM users WHERE key = 'user:1001'"

# Batch file (one query per line, -- comments ignored)
cargo run -- --file queries.txt

# List all supported patterns
cargo run -- --list-patterns
```

### Library Usage

Add to `Cargo.toml`:

```toml
[dependencies]
sql_redis = { git = "https://github.com/allen-munsch/rust-sql-to-nosql" }
```

```rust
use sql_redis::SqlToRedisTransformer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transformer = SqlToRedisTransformer::new()?;

    let cmd = transformer.transform(
        "SELECT * FROM leaderboard__zset WHERE key = 'game:global' AND score > 1000"
    )?;
    println!("{}", cmd);
    // Output: ZRANGEBYSCORE game:global (1000 +inf

    let avg = transformer.transform(
        "SELECT AVG(score) FROM leaderboard__zset WHERE key = 'game:global'"
    )?;
    println!("{}", avg);
    // Output: EVAL 'local members = ...' 1 game:global -inf +inf

    Ok(())
}
```

## BNF Grammar

The full BNF grammar is in [`redis.sql.bnf`](redis.sql.bnf). It defines the mapping from SQL constructs to Redis commands and Lua scripts.

## Limitations

- **Joins, subqueries, CTEs**: infrastructure exists in `pattern/` but not wired to rules
- **Window functions** (RANK, ROW_NUMBER, LAG/LEAD): infrastructure in BNF, not implemented
- **GROUP BY / HAVING**: not implemented
- **LIKE operator**: not supported (requires HSCAN or Lua pattern matching)
- **EVALSHA**: scripts use EVAL (plain-text); production should pre-load via SCRIPT LOAD
- **Nested conditions beyond AND**: OR conditions are partially handled but may produce incorrect results

## Contributing

To add a new pattern:

1. Add a matcher function in `src/pattern/matchers/` (boolean predicate on `&Statement`)
2. Add AST extraction in `src/ast/` if needed
3. Add a context builder in `src/context/` (extracts template variables)
4. Add a template in `src/templates/mod.rs` (or Lua script in `src/lua/mod.rs`)
5. Register the rule in the appropriate `src/rules/` file
6. Place more-specific rules before general ones in the `vec![]`

## License

MIT
