-- @preset_name: 自定义步骤
--- 自定义步骤预设 ---
-- 支持亮度对比度调节的滤镜预处理

function get_controls()
    return {
        max_width = {
            type = "slider",
            label = "最大宽度",
            min = 240,
            max = 800,
            default = 480,
        },
        saturation = {
            type = "slider",
            label = "饱和度",
            min = -100,
            max = 100,
            default = 0,
        },
        sharpness = {
            type = "slider",
            label = "锐化",
            min = 0,
            max = 20,
            default = 0,
        },
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
        crf = {
            type = "slider",
            label = "编码质量",
            min = 0,
            max = 50,
            default = 23,
        },
    }
end

function validate(params, _info)
    local brightness = params["brightness"] or 0
    local contrast = params["contrast"] or 0
    local saturation = params["saturation"] or 0
    local sharpness = params["sharpness"] or 0
    local crf = params["crf"] or 23

    if brightness < -100 or brightness > 100 then
        return { ok = false, error = "亮度范围为 -100 到 100" }
    end
    if contrast < -100 or contrast > 100 then
        return { ok = false, error = "对比度范围为 -100 到 100" }
    end
    if saturation < -100 or saturation > 100 then
        return { ok = false, error = "饱和度范围为 -100 到 100" }
    end
    if sharpness < 0 or sharpness > 20 then
        return { ok = false, error = "锐化范围为 0 到 20" }
    end
    if crf < 0 or crf > 50 then
        return { ok = false, error = "编码质量范围为 0 到 50" }
    end

    return { ok = true }
end

function build_command_pipeline(params, input_path, output_path)
    local brightness = params["brightness"] or 0
    local contrast = params["contrast"] or 0
    local saturation = params["saturation"] or 0
    local sharpness = params["sharpness"] or 0
    local max_width = params["max_width"] or 480
    local crf = params["crf"] or 23

    -- FFmpeg eq 滤镜参数
    local eq_brightness = brightness / 100.0
    local eq_contrast = 1.0 + (contrast / 100.0)
    local eq_saturation = 1.0 + (saturation / 100.0)

    -- 构建滤镜链
    local filters = {}

    -- 缩放
    filters[#filters + 1] = string.format("scale=%d:-1:flags=lanczos", max_width)

    -- eq 滤镜（亮度、对比度、饱和度）
    if brightness ~= 0 or contrast ~= 0 or saturation ~= 0 then
        filters[#filters + 1] = string.format(
            "eq=brightness=%.2f:contrast=%.2f:saturation=%.2f",
            eq_brightness, eq_contrast, eq_saturation
        )
    end

    -- 锐化滤镜
    if sharpness > 0 then
        -- unsharp=luma_msize_x:luma_msize_y:luma_amount
        local amount = sharpness / 10.0
        filters[#filters + 1] = string.format("unsharp=5:5:%.1f", amount)
    end

    local filter_chain = table.concat(filters, ",")

    return {
        {
            desc = "滤镜预处理",
            args = {
                "-i", input_path,
                "-vf", filter_chain,
                "-c:v", "libx264",
                "-preset", "fast",
                "-crf", tostring(crf),
                "-c:a", "aac",
                "-y", output_path,
            },
        },
    }
end
