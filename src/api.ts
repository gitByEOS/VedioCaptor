// Tauri 调用封装：参数校验
import { invoke } from "@tauri-apps/api/core"

export interface ValidateResult { ok: boolean; error?: string }

// 调用后端的 validate 命令
export async function validateParams(preset: string, params: Record<string, any>, inputPath: string): Promise<ValidateResult> {
  return invoke<ValidateResult>("validate", { presetPath: `presets/${preset}.lua`, params, inputPath })
}
