
-- Redis Lua script for GROUP BY operation
-- KEYS[1]: Pattern for KEYS command
-- ARGV[1]: Data type (hash, list, set, zset, string)
-- ARGV[2]: Lua script for grouping key extraction
-- ARGV[3+]: Lua scripts for aggregation functions
-- Returns: Grouped results with aggregations

-- Helper functions
local function hash_to_table(hash_data)
    local result = {}
    for i = 1, #hash_data, 2 do
        result[hash_data[i]] = hash_data[i + 1]
    end
    return result
end

local function get_data(key, data_type)
    if data_type == "hash" then
        return hash_to_table(redis.call('HGETALL', key))
    elseif data_type == "list" then
        return redis.call('LRANGE', key, 0, -1)
    elseif data_type == "set" then
        return redis.call('SMEMBERS', key)
    elseif data_type == "zset" then
        return redis.call('ZRANGEBYSCORE', key, '-inf', '+inf', 'WITHSCORES')
    elseif data_type == "string" then
        return redis.call('GET', key)
    else
        return nil
    end
end

-- Get keys based on pattern
local keys = redis.call('KEYS', KEYS[1])
local data_type = ARGV[1]
local group_key_script = ARGV[2]
local groups = {}

-- Load group key extraction function
local group_key_func = load("return function(key, data) return " .. group_key_script .. " end")()

-- Group data
for _, key in ipairs(keys) do
    local data = get_data(key, data_type)
    local group_key = group_key_func(key, data)

    if not groups[group_key] then
        groups[group_key] = {}
    end
    table.insert(groups[group_key], { key = key, data = data })
end

-- Apply aggregation functions
local results = {}
for group_key, group_data in pairs(groups) do
    local result = { group_key = group_key }

    -- Apply each aggregation function
    for i = 3, #ARGV do
        local agg_func_script = ARGV[i]
        local agg_name = "agg" .. (i - 2)
        local agg_func = load("return function(group_data) return " .. agg_func_script .. " end")()
        result[agg_name] = agg_func(group_data)
    end

    table.insert(results, result)
end

return cjson.encode(results)
