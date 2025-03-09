-- Redis Lua script for ZRANGEBYSCORE operation
-- KEYS[1]: The Redis key of the sorted set
-- ARGV[1]: Min score
-- ARGV[2]: Max score
-- ARGV[3]: Optional 'WITHSCORES' flag (1 to include, 0 to exclude)
-- Returns: Members in the specified score range

local key = KEYS[1]
local min = ARGV[1]
local max = ARGV[2]
local with_scores = tonumber(ARGV[3]) == 1

if with_scores then
    return redis.call('ZRANGEBYSCORE', key, min, max, 'WITHSCORES')
else
    return redis.call('ZRANGEBYSCORE', key, min, max)
end
