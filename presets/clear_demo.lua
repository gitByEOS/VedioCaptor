--- 清晰演示预设 ---
-- 适用于高质量演示场景，单步直接转 GIF

function get_controls()
    return {
        帧率 = {
            type = "slider",
            label = "帧率",
            min = 10,
            max = 60,
            default = 30,
        },
        宽度 = {
            type = "slider",
            label = "宽度",
            min = 640,
            max = 1920,
            default = 1280,
        },
        质量 = {
            type = "select",
            label = "质量",
            values = { "高", "中", "低" },
            default = "高",
        },
    }
end

function validate(_params, _info)
    return { ok = true }
end

function build_command_pipeline(params, input_path, output_path)
    local width = params["宽度"] or 1280
    local fps = params["帧率"] or 30
    local quality = params["质量"] or "高"

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
