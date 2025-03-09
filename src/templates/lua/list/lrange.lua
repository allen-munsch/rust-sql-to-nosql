-- Redis Lua script for LRANGE operation
-- KEYS[1]: The Redis key of the list
-- ARGV[1]: Start index
-- ARGV[2]: Stop index (optional, defaults to -1)
-- Returns: Elements in the specified range

local key = KEYS[1]
local start = tonumber(ARGV[1]) or 0
local stop = tonumber(ARGV[2]) or -1

return redis.call('LRANGE', key, start, stop)
