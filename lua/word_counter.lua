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

-- 需要排除的标点字符（仅在 FILTER_PUNCTUATION = true 时生效）
-- 判断字符是否为标点：ASCII 标点 + CJK 标点范围 + 全角标点
local function is_punctuation_char(codepoint)
    -- ASCII 标点范围
    if codepoint >= 0x0021 and codepoint <= 0x002F then return true end
    if codepoint >= 0x003A and codepoint <= 0x0040 then return true end
    if codepoint >= 0x005B and codepoint <= 0x005E then return true end
    if codepoint == 0x005F or codepoint == 0x0060 then return true end
    if codepoint >= 0x007B and codepoint <= 0x007E then return true end
    -- CJK 符号和标点
    if codepoint >= 0x3000 and codepoint <= 0x303F then return true end
    -- 全角标点
    if codepoint >= 0xFF00 and codepoint <= 0xFFEF then return true end
    -- 通用标点
    if codepoint >= 0x2000 and codepoint <= 0x206F then return true end
    return false
end

-- ========== 核心功能 ==========

-- 解码一个 UTF-8 字符，返回 (codepoint, 占用字节数)
local function decode_utf8_char(text, pos)
    local b1 = text:byte(pos)
    if not b1 then return nil, 0 end

    local cp, n
    if b1 < 0x80 then
        return b1, 1
    elseif b1 < 0xE0 then
        cp = b1 % 0x20
        n = 2
    elseif b1 < 0xF0 then
        cp = b1 % 0x10
        n = 3
    else
        cp = b1 % 0x08
        n = 4
    end

    for j = 1, n - 1 do
        local nb = text:byte(pos + j)
        if not nb then return nil, 0 end
        cp = cp * 64 + (nb % 64)
    end
    return cp, n
end

-- UTF-8 字符计数器（兼容 Lua 5.1）
local function count_chars(text)
    -- 优先使用 Lua 5.3+ utf8.len（不需要过滤标点时）
    if not FILTER_PUNCTUATION then
        local ok, len = pcall(function() return utf8.len(text) end)
        if ok and len then return len end
    end

    -- 手动遍历 UTF-8 字符
    local count = 0
    local i = 1
    while i <= #text do
        local cp, n = decode_utf8_char(text, i)
        if not cp then break end
        if FILTER_PUNCTUATION then
            if not is_punctuation_char(cp) then count = count + 1 end
        else
            count = count + 1
        end
        i = i + n
    end
    return count
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
-- 兼容不同版本 librime-lua 的 API：
--   新版: commit(env)       → env.commit_text 包含上屏文本
--   旧版: commit(env, text) → text 直接传递上屏文本
--   其他: commit(text)      → env 就是字符串
function processor.commit(env, text)
    -- 参数标准化：确保 text 包含上屏字符串
    if type(env) == "string" then
        text = env
    elseif text == nil and type(env) == "table" then
        text = env.commit_text or env.text
    end

    if text == nil or text == "" then
        return
    end

    local word_count = count_chars(text)
    if word_count == nil or word_count == 0 then
        return
    end

    local ok = write_log(word_count)
    if ok then
        io.stderr:write("[word_counter] " .. get_today() .. "," .. word_count .. "\n")
    end
end

return processor
