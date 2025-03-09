-- Redis Lua script for checking multiple members in a set
-- KEYS[1]: The Redis key of the set
-- ARGV: The members to check
-- Returns: Array of 1s and 0s indicating membership

local key = KEYS[1]
local results = {}

for i, member in ipairs(ARGV) do
    table.insert(results, redis.call('SISMEMBER', key, member))
end

return results
