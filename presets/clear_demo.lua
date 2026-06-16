-- @preset_name: 清晰演示
--- 清晰演示预设 ---
-- 适用于高质量演示场景，单步直接转 GIF

function get_controls()
    return {
        fps = {
            type = "slider",
            label = "帧率",
            min = 10,
            max = 60,
            default = 30,
        },
        width = {
            type = "slider",
            label = "宽度",
            min = 640,
            max = 1920,
            default = 1280,
        },
        quality = {
            type = "select",
            label = "质量",
            values = { "高", "中", "低" },
            default = "高",
        },
    }
end

function validate(_params, _info)
    -- 裁剪参数校验
    if _params["crop_w"] then
        if _params["crop_x"] + _params["crop_w"] > _info.width then
            return { ok = false, error = "裁剪区域超出视频宽度" }
        end
        if _params["crop_y"] + _params["crop_h"] > _info.height then
            return { ok = false, error = "裁剪区域超出视频高度" }
        end
        if _params["crop_w"] < 16 or _params["crop_h"] < 16 then
            return { ok = false, error = "裁剪区域至少 16x16" }
        end
    end
    return { ok = true }
end

function build_command_pipeline(params, input_path, output_path)
    local width = params["width"] or 1280
    local fps = params["fps"] or 30
    local quality = params["quality"] or "高"

    -- 质量映射为 CRF 参数
    local crf
    if quality == "高" then
        crf = 10
    elseif quality == "中" then
        crf = 20
    else
        crf = 30
    end

    return {
        {
            desc = "直接转 GIF",
            args = {
                "-i", input_path,
                "-vf", string.format("scale=%d:-1:flags=lanczos,fps=%d", width, fps),
                "-c:v", "gif",
                "-compression_level", tostring(crf),
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
    return { progress = 0, message = "" }
end
