
-- Redis Lua script for inner join between two data sets
-- KEYS[1]: Left pattern for KEYS command
-- KEYS[2]: Right pattern for KEYS command
-- ARGV[1]: Left data type (hash, list, set, zset, string)
-- ARGV[2]: Right data type (hash, list, set, zset, string)
-- ARGV[3]: Lua script for join condition
-- Returns: Array of joined results

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

-- Get keys based on patterns
local left_keys = redis.call('KEYS', KEYS[1])
local right_keys = redis.call('KEYS', KEYS[2])
local left_type = ARGV[1]
local right_type = ARGV[2]
local join_condition = ARGV[3]
local results = {}

-- Load join condition function
local condition_func = load("return function(left_key, left_data, right_key, right_data) return " .. join_condition .. " end")()

-- Perform inner join
for _, left_key in ipairs(left_keys) do
    local left_data = get_data(left_key, left_type)

    for _, right_key in ipairs(right_keys) do
        local right_data = get_data(right_key, right_type)

        -- Check join condition
        if condition_func(left_key, left_data, right_key, right_data) then
            -- Add joined result
            table.insert(results, {
                left_key = left_key,
                left_data = left_data,
                right_key = right_key,
                right_data = right_data
            })
        end
    end
end

return cjson.encode(results)
