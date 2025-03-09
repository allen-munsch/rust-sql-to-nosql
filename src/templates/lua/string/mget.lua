-- Redis Lua script for multiple string GET operations
-- KEYS: The Redis keys to retrieve
-- Returns: Array of values

return redis.call('MGET', unpack(KEYS))
