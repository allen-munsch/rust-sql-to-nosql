
-- Redis Lua script for window functions
-- KEYS[1]: Pattern for KEYS command
-- ARGV[1]: Data type (hash, list, set, zset, string)
-- ARGV[2]: Lua script for partition key extraction
-- ARGV[3]: Lua script for ordering
-- ARGV[4]: Window function type (rank, row_number, lead, lag, etc.)
-- ARGV[5]: Window function arguments
-- Returns: Results with window function applied

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
local partition_key_script = ARGV[2]
local order_script = ARGV[3]
local window_func_type = ARGV[4]
local window_func_args = ARGV[5]
local partitions = {}

-- Load partition key extraction function
local partition_key_func = load("return function(key, data) return " .. partition_key_script .. " end")()

-- Load ordering function
local order_func = load("return function(a, b) return " .. order_script .. " end")()

-- Collect and partition data
for _, key in ipairs(keys) do
    local data = get_data(key, data_type)
    local partition_key = partition_key_func(key, data)

    if not partitions[partition_key] then
        partitions[partition_key] = {}
    end
    table.insert(partitions[partition_key], { key = key, data = data })
end

-- Process each partition
local results = {}
for partition_key, partition_data in pairs(partitions) do
    -- Sort the partition
    table.sort(partition_data, order_func)

    -- Apply window function based on type
    if window_func_type == "rank" then
        -- Implement rank
        local current_rank = 1
        local previous_value = nil

        for i, item in ipairs(partition_data) do
            local value = window_func_args == "" and item.data or 
                          load("return function(data) return " .. window_func_args .. " end")()(item.data)

            if i > 1 and value == previous_value then
                item.rank = partition_data[i-1].rank
            else
                item.rank = current_rank
            end

            previous_value = value
            current_rank = current_rank + 1
            table.insert(results, { key = item.key, data = item.data, rank = item.rank })
        end
    elseif window_func_type == "row_number" then
        -- Implement row_number
        for i, item in ipairs(partition_data) do
            table.insert(results, { key = item.key, data = item.data, row_number = i })
        end
    elseif window_func_type == "lead" or window_func_type == "lag" then
        -- Implement lead/lag
        local offset = tonumber(window_func_args) or 1

        for i, item in ipairs(partition_data) do
            local target_idx = window_func_type == "lead" and i + offset or i - offset
            local result = { key = item.key, data = item.data }

            if target_idx >= 1 and target_idx <= #partition_data then
                result[window_func_type] = partition_data[target_idx].data
            else
                result[window_func_type] = nil
            end

            table.insert(results, result)
        end
    end
end

return cjson.encode(results)
