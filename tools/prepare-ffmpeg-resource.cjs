const fs = require("fs");
const path = require("path");
const ffmpeg = require("@ffmpeg-installer/ffmpeg");

const rootDir = path.resolve(__dirname, "..");
const resourceDir = path.join(rootDir, "src-tauri", "resources");
const targetPath = path.join(resourceDir, "ffmpeg");

fs.mkdirSync(resourceDir, { recursive: true });
fs.copyFileSync(ffmpeg.path, targetPath);

if (process.platform !== "win32") {
  fs.chmodSync(targetPath, 0o755);
}

console.log(`Prepared ffmpeg resource: ${targetPath}`);
