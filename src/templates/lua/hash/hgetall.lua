-- Redis Lua script for HGETALL operation
-- KEYS[1]: The Redis key of the hash
-- Returns: The hash fields and values as a table

local key = KEYS[1]
local result = redis.call('HGETALL', key)

-- Convert Redis response to Lua table
local table_result = {}
for i = 1, #result, 2 do
    table_result[result[i]] = result[i + 1]
end

return table_result
