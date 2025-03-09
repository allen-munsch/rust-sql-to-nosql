-- Redis Lua script for HMGET operation
-- KEYS[1]: The Redis key of the hash
-- ARGV: The fields to retrieve
-- Returns: Array of values for the specified fields

local key = KEYS[1]
return redis.call('HMGET', key, unpack(ARGV))
