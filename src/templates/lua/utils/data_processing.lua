
-- Data processing utility functions for Redis data

-- Convert Redis hash format to Lua table
local function hash_to_table(hash_data)
    if not hash_data or #hash_data == 0 then
        return {}
    end

    local result = {}
    for i = 1, #hash_data, 2 do
        result[hash_data[i]] = hash_data[i + 1]
    end
    return result
end

-- Extract a field from Redis hash data
local function extract_field(hash_data, field_name)
    if not hash_data or #hash_data == 0 then
        return nil
    end

    for i = 1, #hash_data, 2 do
        if hash_data[i] == field_name then
            return hash_data[i + 1]
        end
    end
    return nil
end

-- Convert Redis sorted set format to Lua table
local function zset_to_table(zset_data)
    if not zset_data or #zset_data == 0 then
        return {}
    end

    local result = {}
    for i = 1, #zset_data, 2 do
        result[zset_data[i]] = tonumber(zset_data[i + 1])
    end
    return result
end

-- Convert Redis list to Lua table with indices
local function list_to_indexed_table(list_data)
    if not list_data or #list_data == 0 then
        return {}
    end

    local result = {}
    for i, value in ipairs(list_data) do
        result[i-1] = value  -- Redis uses 0-based indexing
    end
    return result
end

-- Deep copy a table
local function deep_copy(obj)
    if type(obj) ~= 'table' then
        return obj
    end

    local res = {}
    for k, v in pairs(obj) do
        res[k] = type(v) == 'table' and deep_copy(v) or v
    end

    return res
end

-- Merge two tables
local function merge_tables(t1, t2)
    local result = deep_copy(t1)

    for k, v in pairs(t2) do
        result[k] = v
    end

    return result
end

-- Filter a table based on a predicate function
local function filter_table(tbl, predicate)
    local result = {}

    for k, v in pairs(tbl) do
        if predicate(k, v) then
            result[k] = v
        end
    end

    return result
end

-- Return the functions
return {
    hash_to_table = hash_to_table,
    extract_field = extract_field,
    zset_to_table = zset_to_table,
    list_to_indexed_table = list_to_indexed_table,
    deep_copy = deep_copy,
    merge_tables = merge_tables,
    filter_table = filter_table
}
