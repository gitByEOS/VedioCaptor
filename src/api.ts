// Tauri 调用封装：参数校验
import { invoke } from "@tauri-apps/api/core"

export interface ValidateResult { ok: boolean; error?: string }
export interface ConversionResult { output_path: string; message?: string; file_info?: string }
export interface VideoInfo { duration: number; width: number; height: number }
export interface PreviewVideo { preview_path: string; video_info: VideoInfo }

// 为浏览器生成可播放的 MP4 预览
export async function prepareVideoPreview(inputPath: string, startSec: number, endSec: number): Promise<PreviewVideo> {
  return invoke<PreviewVideo>("prepare_video_preview", { inputPath, startSec, endSec })
}

// 调用后端的 validate 命令
export async function validateParams(preset: string, params: Record<string, any>, videoInfo: VideoInfo): Promise<ValidateResult> {
  return invoke<ValidateResult>("validate", { presetName: preset, params, videoInfo })
}

// 调用后端的 execute_conversion 命令
export async function executeConversion(
  presetName: string,
  params: Record<string, any>,
  inputPath: string,
  startTime: string,
  endTime: string,
  outputPath: string,
): Promise<ConversionResult> {
  return invoke<ConversionResult>("execute_conversion", {
    presetName,
    params,
    inputPath,
    startTime,
    endTime,
    outputPath,
  })
}
