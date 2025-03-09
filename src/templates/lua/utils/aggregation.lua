
-- Aggregation utility functions for SQL-like operations

-- Calculate sum of values
local function sum(values, field_name)
    local total = 0
    for _, v in ipairs(values) do
        local value = field_name and v[field_name] or v
        if type(value) == 'number' then
            total = total + value
        elseif type(value) == 'string' then
            local num = tonumber(value)
            if num then
                total = total + num
            end
        end
    end
    return total
end

-- Calculate average of values
local function avg(values, field_name)
    local count = #values
    if count == 0 then
        return nil
    end
    return sum(values, field_name) / count
end

-- Find minimum value
local function min(values, field_name)
    if #values == 0 then
        return nil
    end

    local min_val = nil
    for _, v in ipairs(values) do
        local value = field_name and v[field_name] or v
        if type(value) == 'number' or type(value) == 'string' and tonumber(value) then
            if type(value) == 'string' then 
                value = tonumber(value) 
            end

            if min_val == nil or value < min_val then
                min_val = value
            end
        end
    end

    return min_val
end

-- Find maximum value
local function max(values, field_name)
    if #values == 0 then
        return nil
    end

    local max_val = nil
    for _, v in ipairs(values) do
        local value = field_name and v[field_name] or v
        if type(value) == 'number' or type(value) == 'string' and tonumber(value) then
            if type(value) == 'string' then 
                value = tonumber(value) 
            end

            if max_val == nil or value > max_val then
                max_val = value
            end
        end
    end

    return max_val
end

-- Count values
local function count(values, field_name)
    if not field_name then
        return #values
    end

    local count_val = 0
    for _, v in ipairs(values) do
        if v[field_name] ~= nil then
            count_val = count_val + 1
        end
    end

    return count_val
end

-- Count distinct values
local function count_distinct(values, field_name)
    local unique = {}
    for _, v in ipairs(values) do
        local value = field_name and v[field_name] or v
        if value ~= nil then
            unique[tostring(value)] = true
        end
    end

    local count_val = 0
    for _ in pairs(unique) do
        count_val = count_val + 1
    end

    return count_val
end

-- Return the functions
return {
    sum = sum,
    avg = avg,
    min = min,
    max = max,
    count = count,
    count_distinct = count_distinct
}
