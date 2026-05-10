// Tauri 调用封装：参数校验
import { invoke } from "@tauri-apps/api/core"

export interface ValidateResult { ok: boolean; error?: string }
export interface ConversionResult { output_path: string; message?: string }

// 调用后端的 validate 命令
export async function validateParams(preset: string, params: Record<string, any>, inputPath: string): Promise<ValidateResult> {
  return invoke<ValidateResult>("validate", { presetPath: `presets/${preset}.lua`, params, inputPath })
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
