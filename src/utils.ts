// 时间与路径工具函数

/**
 * HH:MM:SS 转秒数
 */
export function timeToSeconds(hhmmss: string): number {
  const parts = hhmmss.split(":");
  if (parts.length !== 3) return 0;
  const h = parseInt(parts[0], 10) || 0;
  const m = parseInt(parts[1], 10) || 0;
  const s = parseInt(parts[2], 10) || 0;
  return h * 3600 + m * 60 + s;
}

/**
 * 秒数转 HH:MM:SS
 */
export function secondsToTime(totalSeconds: number): string {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  return [h, m, s].map(v => String(v).padStart(2, "0")).join(":");
}

/**
 * 校验时间范围：结束时间必须大于起始时间
 */
export function isValidTimeRange(start: string, end: string): boolean {
  return timeToSeconds(end) > timeToSeconds(start);
}

/**
 * 从完整路径中提取文件名
 */
export function extractFileName(path: string): string {
  return path.replace(/\\/g, "/").split("/").pop() || path;
}

/**
 * 从完整路径中提取不含扩展名的文件名
 */
export function extractBaseName(path: string): string {
  const name = extractFileName(path);
  return name.replace(/\.[^.]+$/, "");
}
