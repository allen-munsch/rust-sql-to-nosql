
-- Type conversion utility functions for SQL operations

-- Convert to number
local function to_number(value)
    if value == nil then
        return nil
    end
    return tonumber(value)
end

-- Convert to boolean
local function to_boolean(value)
    if value == nil then
        return nil
    elseif value == 0 or value == '0' or value == '' or value == 'false' or value == 'FALSE' then
        return false
    else
        return true
    end
end

-- Convert to string
local function to_string(value)
    if value == nil then
        return nil
    end
    return tostring(value)
end

-- Parse ISO date string (YYYY-MM-DD)
local function parse_date(date_str)
    if date_str == nil then
        return nil
    end

    local year, month, day = date_str:match("(%d+)-(%d+)-(%d+)")
    if not year then
        return nil
    end

    -- Use os.time to convert to timestamp (seconds since epoch)
    return os.time({year=tonumber(year), month=tonumber(month), day=tonumber(day)})
end

-- Parse ISO timestamp string (YYYY-MM-DD HH:MM:SS)
local function parse_timestamp(timestamp_str)
    if timestamp_str == nil then
        return nil
    end

    local year, month, day, hour, min, sec = timestamp_str:match("(%d+)-(%d+)-(%d+) (%d+):(%d+):(%d+)")
    if not year then
        return nil
    end

    -- Use os.time to convert to timestamp (seconds since epoch)
    return os.time({year=tonumber(year), month=tonumber(month), day=tonumber(day), 
                  hour=tonumber(hour), min=tonumber(min), sec=tonumber(sec)})
end

-- Format timestamp as ISO date string
local function format_date(timestamp)
    if timestamp == nil then
        return nil
    end

    return os.date("%Y-%m-%d", tonumber(timestamp))
end

-- Format timestamp as ISO timestamp string
local function format_timestamp(timestamp)
    if timestamp == nil then
        return nil
    end

    return os.date("%Y-%m-%d %H:%M:%S", tonumber(timestamp))
end

-- Return the functions
return {
    to_number = to_number,
    to_boolean = to_boolean,
    to_string = to_string,
    parse_date = parse_date,
    parse_timestamp = parse_timestamp,
    format_date = format_date,
    format_timestamp = format_timestamp
}
