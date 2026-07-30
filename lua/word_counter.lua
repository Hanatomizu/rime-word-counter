--[[
word_counter.lua — Rime 字数统计日志
每次上屏记录 (YYYY-MM-DD,字数) 到 ~/.cache/rime-word-counter/rime_word.log
--]]

local LOG_PATH = "~/.cache/rime-word-counter/rime_word.log"
local FILTER_PUNCTUATION = false

local function is_punctuation_char(cp)
    if cp >= 0x0021 and cp <= 0x002F then return true end
    if cp >= 0x003A and cp <= 0x0040 then return true end
    if cp >= 0x005B and cp <= 0x005E then return true end
    if cp == 0x005F or cp == 0x0060 then return true end
    if cp >= 0x007B and cp <= 0x007E then return true end
    if cp >= 0x3000 and cp <= 0x303F then return true end
    if cp >= 0xFF00 and cp <= 0xFFEF then return true end
    if cp >= 0x2000 and cp <= 0x206F then return true end
    return false
end

local function decode_utf8_char(text, pos)
    local b1 = text:byte(pos)
    if not b1 then return nil, 0 end
    local cp, n
    if b1 < 0x80 then return b1, 1
    elseif b1 < 0xE0 then cp, n = b1 % 0x20, 2
    elseif b1 < 0xF0 then cp, n = b1 % 0x10, 3
    else cp, n = b1 % 0x08, 4 end
    for j = 1, n - 1 do
        local nb = text:byte(pos + j)
        if not nb then return nil, 0 end
        cp = cp * 64 + (nb % 64)
    end
    return cp, n
end

local function count_chars(text)
    if not FILTER_PUNCTUATION then
        local ok, len = pcall(function() return utf8.len(text) end)
        if ok and len then return len end
    end
    local count = 0
    local i = 1
    while i <= #text do
        local cp, n = decode_utf8_char(text, i)
        if not cp then break end
        if FILTER_PUNCTUATION then
            if not is_punctuation_char(cp) then count = count + 1 end
        else count = count + 1 end
        i = i + n
    end
    return count
end

local function expand_path(path)
    if path:sub(1, 1) == "~" then
        local home = os.getenv("HOME") or os.getenv("USERPROFILE")
        if home then return home .. path:sub(2) end
    end
    return path
end

local function write_log(word_count)
    local path = expand_path(LOG_PATH)
    local dir = path:match("^(.*/)")
    if dir and dir ~= "" then os.execute("mkdir -p " .. dir) end
    local file, err = io.open(path, "a")
    if not file then
        io.stderr:write("[word_counter] 无法打开日志文件: " .. (err or "unknown") .. "\n")
        return
    end
    file:write(os.date("%Y-%m-%d") .. "," .. word_count .. "\n")
    file:close()
end

-- ========== Rime Processor ==========

local processor = {}

function processor.init(env)
    local path = expand_path(LOG_PATH)
    io.stderr:write("[word_counter] 已加载，日志路径: " .. path .. "\n")

    -- 使用 commit_notifier 监听文本上屏事件
    local ctx = env.engine.context
    if ctx.commit_notifier then
        ctx.commit_notifier:connect(function(ctx2)
            local text = ctx2:get_commit_text()
            if text and #text > 0 then
                local wc = count_chars(text)
                if wc and wc > 0 then
                    write_log(wc)
                end
            end
        end)
    end
end

function processor.func(key, env)
    return 2  -- kNoop
end

return processor
