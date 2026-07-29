--[[
word_counter.lua — Rime 输入法字数统计日志记录脚本

功能：
  每次上屏时记录当前日期和字数到 CSV 日志文件。
  配合 Rust 编写的 rime-word-counter 程序进行数据汇总和可视化。

部署步骤：
  1. 将此脚本放入 Rime 用户目录下的 lua/ 文件夹：
       Linux/macOS:   ~/.config/fcitx/rime/lua/
                      ~/Library/Rime/lua/
       Windows:       %APPDATA%\Rime\lua\
  2. 在 rime.lua 中注册 processor：
       word_counter_processor = require("word_counter")
  3. 在所需输入方案的 schema 中添加 processor：
       schema:
         ...
         engine:
           processors:
             - ascii_composer
             - recognizer
             - key_binder
             - speller
             - punctuator
             - selector
             - navigator
             - express_editor
             - word_counter_processor    ← 添加此行（建议放在最后）
  4. 重新部署 Rime 即可生效。

日志路径：
  默认写入 ~/.cache/rime-word-counter/rime_word.log
  可通过修改下面的 LOG_PATH 变量自定义。
--]]

-- ========== 配置区域 ==========

-- 日志文件路径（支持 ~ 开头表示用户主目录）
local LOG_PATH = "~/.cache/rime-word-counter/rime_word.log"

-- 是否过滤标点符号（true = 只统计汉字和字母，false = 统计所有字符）
local FILTER_PUNCTUATION = false

-- 需要排除的标点字符（Unicode 范围，仅在 FILTER_PUNCTUATION = true 时生效）
-- 可以根据需要增删
local PUNCTUATION_PATTERN = "[\
    \\u3000-\\u303F\\\  -- CJK 符号和标点\
    \\uFF00-\\uFFEF\\\  -- 全角标点\
    \\u0021-\\u002F\\\  -- 半角标点 !\"#$%&'()*+,-./\
    \\u003A-\\u0040\\\  -- :;<=>?@\
    \\u005B-\\u005E\\\  -- [\\]^\
    \\u005F\\u0060\\\   -- _`\
    \\u007B-\\u007E\\\  -- {|}~\
    \\u2000-\\u206F\\\  -- 通用标点\
]"

-- ========== 核心功能 ==========

-- 兼容 Lua 5.1 的 UTF-8 字符计数器
-- 如果环境提供了 utf8.len，优先使用
local function count_chars(text)
    -- 尝试使用 Lua 5.3+ 内置 utf8 库
    local ok, result = pcall(function()
        if FILTER_PUNCTUATION then
            -- 过滤标点后统计
            local clean = text:gsub(PUNCTUATION_PATTERN, "")
            return utf8.len(clean)
        else
            return utf8.len(text)
        end
    end)

    if ok then
        return result
    end

    -- 备用方案：手动遍历 UTF-8 字节序列（兼容 Lua 5.1）
    local count = 0
    local i = 1
    while i <= #text do
        local byte = text:byte(i)
        if byte < 128 then
            -- 单字节 ASCII
            if not FILTER_PUNCTUATION then
                count = count + 1
            elseif not is_ascii_punctuation(byte) then
                count = count + 1
            end
            i = i + 1
        elseif byte < 224 then
            -- 双字节（如拉丁扩展字符）
            count = count + 1
            i = i + 2
        elseif byte < 240 then
            -- 三字节（CJK 统一表意文字等）
            count = count + 1
            i = i + 3
        else
            -- 四字节（表情符号等）
            count = count + 1
            i = i + 4
        end
    end
    return count
end

-- 判断 ASCII 标点（仅当 FILTER_PUNCTUATION 启用时使用）
local function is_ascii_punctuation(byte)
    local punct_ranges = {
        {0x21, 0x2F}, {0x3A, 0x40},
        {0x5B, 0x5E}, {0x60, 0x60},
        {0x7B, 0x7E},
    }
    for _, range in ipairs(punct_ranges) do
        if byte >= range[1] and byte <= range[2] then
            return true
        end
    end
    return false
end

-- 展开路径（将开头的 ~ 替换为用户主目录）
local function expand_path(path)
    if path:sub(1, 1) == "~" then
        local home = os.getenv("HOME")
                or os.getenv("USERPROFILE")
                or os.getenv("HOMEDRIVE") .. (os.getenv("HOMEPATH") or "")
        if home then
            return home .. path:sub(2)
        end
    end
    return path
end

-- 获取当前日期字符串（格式 YYYY-MM-DD）
local function get_today()
    return os.date("%Y-%m-%d")
end

-- 写入日志：追加一行 "YYYY-MM-DD,字数"
local function write_log(word_count)
    local path = expand_path(LOG_PATH)
    local dir = path:match("^(.*/)")
        or path:match("^(.*\\)")  -- Windows 路径分隔符
        or ""

    -- 确保目录存在
    if dir ~= "" then
        os.execute("mkdir -p " .. dir)  -- Unix
        -- Windows 下可使用: os.execute("mkdir " .. dir:gsub("/", "\\"))
    end

    -- 以追加模式打开文件
    local file, err = io.open(path, "a")
    if not file then
        -- 写入错误到 Rime 日志（可通过 Rime 的日志查看）
        io.stderr:write("[word_counter] 无法打开日志文件: " .. (err or "unknown") .. "\n")
        return false
    end

    -- 写入 CSV 行：日期,字数
    local line = get_today() .. "," .. word_count .. "\n"
    file:write(line)
    file:close()
    return true
end

-- ========== Rime 处理器接口 ==========

local processor = {}

-- 处理器初始化（Rime 在加载时调用）
function processor.init(env)
    -- 可在此处进行初始化检查
    local path = expand_path(LOG_PATH)
    io.stderr:write("[word_counter] 已加载，日志路径: " .. path .. "\n")
end

-- 按键处理（必须返回，保持不中断输入）
-- Rime 调用 func 处理每个按键事件，返回 1 表示不处理（noop）
function processor.func(key, env)
    -- 我们不需要拦截按键，返回 1 让 Rime 继续正常处理
    return 1  -- kNoop
end

-- 上屏回调（Rime 在有文本上屏时调用）
-- text: 上屏的完整文本字符串
function processor.commit(env, text)
    if text == nil or text == "" then
        return
    end

    local word_count = count_chars(text)
    if word_count == nil or word_count == 0 then
        -- 上屏内容为空或全为标点时可以选择忽略
        return
    end

    -- 检查是否为功能键产生的空内容
    if type(text) == "string" and text ~= "" then
        local ok = write_log(word_count)
        if ok then
            -- 调试信息（可通过 Rime 日志查看）
            -- io.stderr:write("[word_counter] 记录: " .. get_today() .. "," .. word_count .. "\n")
        end
    end
end

return processor
