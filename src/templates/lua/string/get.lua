-- Redis Lua script for string GET operation
-- KEYS[1]: The Redis key to retrieve
-- Returns: The string value

local key = KEYS[1]
return redis.call('GET', key)
