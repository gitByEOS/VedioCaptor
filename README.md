# VideoCaptor

我是做游戏开发的，经常需要给策划看一些效果表现，为了方便自己写了这个工具，顺便开源了。

还有就是测试AI在这套框架下的能力，以及 Lua 当大脑 Rust 当手脚的可行性验证。

## 功能概述
视频转 GIF 工具，支持多档位预设、动态参数调节、实时进度反馈。

- 拖拽或选择视频文件
- 自定义时间范围裁剪
- 快速一键导出 GIF
- 最快只需3步操作

## 项目架构

### 语言分工

| 语言 | 占比 | 职责 | 不做的事 |
|------|------|------|----------|
| **Lua** | ~40% | 流程编排、参数校验、ffmpeg 命令构建、进度解析、后处理 | 不碰文件系统、不启动进程 |
| **Rust** | ~40% | Lua 虚拟机、ffmpeg 进程管理、Tauri commands 桥接、事件推送 | 不做流程决策、不拼命令逻辑 |
| **TypeScript/Vue** | ~20% | 表单渲染、进度展示、状态管理、用户交互 | 不含业务逻辑、不含 ffmpeg 知识 |

### 设计原则

**Lua 主权**：Lua 是"总指挥"，Rust 是"执行代理"，前端是"展示终端"。

```
用户点击 [生成 GIF]
→ 前端 invoke('execute_conversion', {preset, params, input, start, end, output})
→ Rust 加载 Lua 预设，调用 validate(params, video_info)
  → 若失败，直接返回错误给前端
→ Rust 调用 build_command_pipeline(...) 得到步骤数组
→ Rust 依次启动 ffmpeg，每收到 stderr 行调用 parse_progress(...)
  → 进度事件 conversion-progress 推送给前端
→ 所有步骤完成后，调用 on_complete(output, params)
  → 执行后处理，返回结果给前端

全程 Rust 不知道步骤有几个、进度如何计算，完全交由 Lua 决策。
```

### 目录结构

```
src/
├── App.vue              # 主入口：状态机 (idle → validating → converting → done/error)
├── api.ts               # Tauri invoke 封装 (validateParams, executeConversion)
├── utils.ts             # 工具函数 (timeToSeconds, secondsToTime, isValidTimeRange)
└── components/
    ├── FileSelector.vue  # 文件选择 + 视频预览
    ├── TimeRangeSlider.vue # 时间范围拖动选择
    ├── PresetParamPanel.vue # 预设选择 + 动态参数面板
    ├── ProgressView.vue  # 实时进度条 + 步骤日志
    └── ResultView.vue    # GIF 预览 + 导出

src-tauri/src/
├── lib.rs               # Tauri 初始化 + 事件监听 + 命令注册
├── commands.rs          # scan_presets, get_controls, validate
├── commands_presets.rs  # resolve_presets_dir, list_presets, get_preset_controls
├── execute_commands.rs  # execute_conversion (完整管线执行)
├── ffmpeg_commands.rs   # run_ffmpeg, run_pipeline
├── ffmpeg_runner.rs     # FFmpeg 进程管理 (启动/捕获stderr/终止)
├── lua_runtime.rs       # Lua 虚拟机：加载预设、调用函数、JSON⇄Lua 转换
├── pipeline_executor.rs # 管线步骤执行 + 进度事件推送
└── types.rs             # 类型定义 (ControlDef, PresetInfo, ProgressEvent 等)

presets/
├── clear_demo.lua       # 清晰演示预设 (单步直接转 GIF)
├── custom_steps.lua     # 滤镜预设 (亮度/对比度/饱和度/锐化调节)
└── emoji_make.lua       # 表情制作预设
```

## 数据流

1. 选择视频文件 → 显示预览
2. 选择 Lua 预设 → 动态加载控件定义
3. 调整参数 → 实时渲染对应控件
4. 点击「生成 GIF」→ 校验时间范围 + 参数合法性
5. 校验通过 → 调用 execute_conversion → 推送 conversion-progress 事件
6. 进度更新 → ProgressView 实时更新
7. 完成 → ResultView 展示 GIF + 导出到文件夹

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust (Tauri 2) + Lua 5.4 (mlua)
- **平台**: macOS (Apple Silicon)
- **依赖**: FFmpeg

## 开源协议

Apache License 2.0
