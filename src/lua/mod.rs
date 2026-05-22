// lua/mod.rs — Redis Lua scripting support for complex SQL operations
//
// Provides Lua script templates for aggregate functions, statistical operations,
// and other features that require server-side processing in Redis.

/// A Lua-based Redis EVAL command ready for execution
pub struct EvalCommand {
    pub script: String,
    pub keys: Vec<String>,
    pub args: Vec<String>,
}

impl EvalCommand {
    /// Format as an EVAL command string
    pub fn to_redis_command(&self) -> String {
        let script_quoted = self.script.replace('\'', "''");
        let parts = std::iter::once(format!("EVAL '{}' {}", script_quoted, self.keys.len()))
            .chain(self.keys.iter().map(|k| k.clone()))
            .chain(self.args.iter().map(|a| a.clone()));
        parts.collect::<Vec<_>>().join(" ")
    }
}

// ============================================================
// ZSet Aggregate Scripts
// ============================================================

/// AVG(score) — compute average score over a zset range
pub const ZSET_AVG_SCRIPT: &str = r#"
local members = redis.call('ZRANGEBYSCORE', KEYS[1], ARGV[1], ARGV[2], 'WITHSCORES')
local sum = 0
local count = 0
for i = 1, #members, 2 do
    local score = tonumber(members[i + 1])
    if score then
        sum = sum + score
        count = count + 1
    end
end
if count > 0 then
    return sum / count
else
    return nil
end
"#;

/// SUM(score) — compute total score over a zset range
pub const ZSET_SUM_SCRIPT: &str = r#"
local members = redis.call('ZRANGEBYSCORE', KEYS[1], ARGV[1], ARGV[2], 'WITHSCORES')
local sum = 0
for i = 1, #members, 2 do
    local score = tonumber(members[i + 1])
    if score then
        sum = sum + score
    end
end
return sum
"#;

/// MIN(score) — find minimum score over a zset range
pub const ZSET_MIN_SCRIPT: &str = r#"
local members = redis.call('ZRANGEBYSCORE', KEYS[1], ARGV[1], ARGV[2], 'WITHSCORES')
if #members == 0 then
    return nil
end
return members[2]
"#;

/// MAX(score) — find maximum score over a zset range
pub const ZSET_MAX_SCRIPT: &str = r#"
local members = redis.call('ZRANGEBYSCORE', KEYS[1], ARGV[1], ARGV[2], 'WITHSCORES')
if #members == 0 then
    return nil
end
return members[#members]
"#;

/// STDDEV_POP(score) — population standard deviation over a zset range
pub const ZSET_STDDEV_POP_SCRIPT: &str = r#"
local members = redis.call('ZRANGEBYSCORE', KEYS[1], ARGV[1], ARGV[2], 'WITHSCORES')
local sum = 0
local sum_sq = 0
local count = 0
for i = 1, #members, 2 do
    local score = tonumber(members[i + 1])
    if score then
        sum = sum + score
        sum_sq = sum_sq + score * score
        count = count + 1
    end
end
if count == 0 then
    return nil
end
local mean = sum / count
return math.sqrt((sum_sq / count) - (mean * mean))
"#;

// ============================================================
// Hash Aggregate Scripts (single-key hash operations)
// ============================================================

/// AVG of a specific hash field (e.g., AVG(salary) FROM employees__hash WHERE key = 'dept:1')
/// Actually computes the value of the field (single-hash aggregate returns the value itself)
pub const HASH_FIELD_AVG_SCRIPT: &str = r#"
-- For a single hash key, AVG(field) returns the field value (single value avg = value)
local val = redis.call('HGET', KEYS[1], ARGV[1])
if val then
    local num = tonumber(val)
    if num then
        return num
    end
end
return nil
"#;

/// SUM of a specific hash field (single hash: returns the field value)
pub const HASH_FIELD_SUM_SCRIPT: &str = r#"
local val = redis.call('HGET', KEYS[1], ARGV[1])
if val then
    local num = tonumber(val)
    if num then
        return num
    end
end
return 0
"#;

/// MIN of a specific hash field (single hash: returns the field value)
pub const HASH_FIELD_MIN_SCRIPT: &str = r#"
local val = redis.call('HGET', KEYS[1], ARGV[1])
if val then
    local num = tonumber(val)
    if num then
        return num
    end
end
return nil
"#;

/// MAX of a specific hash field (single hash: returns the field value)
pub const HASH_FIELD_MAX_SCRIPT: &str = r#"
local val = redis.call('HGET', KEYS[1], ARGV[1])
if val then
    local num = tonumber(val)
    if num then
        return num
    end
end
return nil
"#;

// ============================================================
// List Aggregate Scripts
// ============================================================

/// AVG of list values (iterate list, convert to numbers, compute average)
pub const LIST_AVG_SCRIPT: &str = r#"
local members = redis.call('LRANGE', KEYS[1], 0, -1)
if #members == 0 then
    return nil
end
local sum = 0
local count = 0
for i = 1, #members do
    local num = tonumber(members[i])
    if num then
        sum = sum + num
        count = count + 1
    end
end
if count > 0 then
    return sum / count
else
    return nil
end
"#;

/// SUM of list values
pub const LIST_SUM_SCRIPT: &str = r#"
local members = redis.call('LRANGE', KEYS[1], 0, -1)
local sum = 0
for i = 1, #members do
    local num = tonumber(members[i])
    if num then
        sum = sum + num
    end
end
return sum
"#;

/// MIN of list values
pub const LIST_MIN_SCRIPT: &str = r#"
local members = redis.call('LRANGE', KEYS[1], 0, -1)
if #members == 0 then
    return nil
end
local min_val = nil
for i = 1, #members do
    local num = tonumber(members[i])
    if num then
        if min_val == nil or num < min_val then
            min_val = num
        end
    end
end
return min_val
"#;

/// MAX of list values
pub const LIST_MAX_SCRIPT: &str = r#"
local members = redis.call('LRANGE', KEYS[1], 0, -1)
if #members == 0 then
    return nil
end
local max_val = nil
for i = 1, #members do
    local num = tonumber(members[i])
    if num then
        if max_val == nil or num > max_val then
            max_val = num
        end
    end
end
return max_val
"#;

// ============================================================
// Convenience builders
// ============================================================

/// Build a zset aggregate EVAL command
pub fn zset_aggregate(script: &str, key: &str, min: &str, max: &str) -> String {
    EvalCommand {
        script: script.to_string(),
        keys: vec![key.to_string()],
        args: vec![min.to_string(), max.to_string()],
    }
    .to_redis_command()
}

/// Build a hash aggregate EVAL command (operates on a single hash key)
pub fn hash_aggregate(script: &str, key: &str, fields: &[String]) -> String {
    EvalCommand {
        script: script.to_string(),
        keys: vec![key.to_string()],
        args: fields.to_vec(),
    }
    .to_redis_command()
}

/// Build a list aggregate EVAL command
pub fn list_aggregate(script: &str, key: &str) -> String {
    EvalCommand {
        script: script.to_string(),
        keys: vec![key.to_string()],
        args: vec![],
    }
    .to_redis_command()
}
