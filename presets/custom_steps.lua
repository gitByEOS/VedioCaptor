--- 自定义步骤预设 ---
-- 支持亮度对比度调节的滤镜预处理

function get_controls()
    return {
        brightness = {
            type = "slider",
            label = "亮度",
            min = -100,
            max = 100,
            default = 0,
        },
        contrast = {
            type = "slider",
            label = "对比度",
            min = -100,
            max = 100,
            default = 0,
        },
    }
end

function validate(params, _info)
    local brightness = params["brightness"] or 0
    local contrast = params["contrast"] or 0

    if brightness < -100 or brightness > 100 then
        return { ok = false, error = "亮度范围为 -100 到 100" }
    end
    if contrast < -100 or contrast > 100 then
        return { ok = false, error = "对比度范围为 -100 到 100" }
    end

    return { ok = true }
end

function build_command_pipeline(params, input_path, output_path)
    local brightness = params["brightness"] or 0
    local contrast = params["contrast"] or 0

    -- FFmpeg eq 滤镜参数: brightness=-1.0~1.0, contrast=0.0~2.0
    local eq_brightness = brightness / 100.0
    local eq_contrast = 1.0 + (contrast / 100.0)

    local filter_expr = string.format(
        "eq=brightness=%.2f:contrast=%.2f",
        eq_brightness, eq_contrast
    )

    return {
        {
            desc = "滤镜预处理（亮度对比度）",
            args = {
                "-i", input_path,
                "-vf", filter_expr,
                "-c:v", "libx264",
                "-preset", "fast",
                "-crf", "18",
                "-y", output_path,
            },
        },
    }
end
