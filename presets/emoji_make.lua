-- @preset_name: 表情制作
--- 表情小文件预设 ---
-- 适用于社交分享的场景，输出为调色板优化的 GIF

function get_controls()
    return {
        max_width = {
            type = "slider",
            label = "最大宽度",
            min = 240,
            max = 800,
            default = 480,
        },
        fps = {
            type = "slider",
            label = "帧率",
            min = 5,
            max = 30,
            default = 10,
        },
        quality = {
            type = "select",
            label = "质量",
            values = { "小文件", "均衡", "最高" },
            default = "小文件",
        },
    }
end

function validate(params, _info)
    if _info.duration > 15 and params["fps"] and params["fps"] > 15 then
        return { ok = false, error = "长视频建议降低帧率" }
    end
    return { ok = true }
end

function build_command_pipeline(params, input_path, output_path)
    local width = params["max_width"] or 480
    local fps = params["fps"] or 10
    local quality = params["quality"] or "小文件"

    -- 质量映射为调色板参数
    local palette_colors
    if quality == "最高" then
        palette_colors = 256
    elseif quality == "均衡" then
        palette_colors = 128
    else
        palette_colors = 64
    end

    local palette_path = output_path .. ".palette.png"

    return {
        {
            desc = "生成调色板",
            args = {
                "-i", input_path,
                "-vf", string.format("scale=%d:-1:flags=lanczos,palettegen=max_colors=%d", width, palette_colors),
                "-y", palette_path,
            },
        },
        {
            desc = "使用调色板生成 GIF",
            args = {
                "-i", input_path,
                "-i", palette_path,
                "-lavfi", string.format("scale=%d:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer", width),
                "-r", tostring(fps),
                "-y", output_path,
            },
        },
    }
end

-- 解析 FFmpeg 进度输出
function parse_progress(line, step_index, step_name, duration_sec)
    -- 匹配 time= 格式: time=00:01:23.45 或 time=123.45
    local time_match = line:match("time=(%d+:%d+:%d+%.?%d*)")
    if time_match then
        local h, m, s = time_match:match("(%d+):(%d+):(%d+%.?%d*)")
        if h and m and s then
            local current_sec = tonumber(h) * 3600 + tonumber(m) * 60 + tonumber(s)
            if duration_sec > 0 then
                local pct = (current_sec / duration_sec) * 100
                return { progress = math.min(pct, 100), message = string.format("%.1f%% (%.1fs/%.1fs)", pct, current_sec, duration_sec) }
            end
        end
    end

    -- 匹配 frame= 格式作为备选
    local frame_match = line:match("frame=%s*(%d+)")
    if frame_match then
        local frame = tonumber(frame_match)
        return { progress = 0, message = string.format("帧 %d", frame) }
    end

    return { progress = 0, message = "" }
end
