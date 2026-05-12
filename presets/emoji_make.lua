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
