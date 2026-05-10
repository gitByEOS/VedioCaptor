# VideoCaptor (agent-j)

Lua主控视频转GIF工具：Rust核心 + Tauri前端 + Lua预设脚本。Lua负责流程编排、参数校验、命令构建、进度解析、后处理，Rust仅做执行代理。

## 项目架构

### 语言分工

| 语言 | 占比 | 职责 | 不做的事 |
|------|------|------|----------|
| **Lua** | ~40% | 流程编排、参数校验、命令构建、进度解析策略、后处理 | 不碰文件系统、不启动进程 |
| **Rust** | ~40% | Lua 虚拟机、ffmpeg 进程管理、事件推送、Tauri commands 桥接 | 不做流程决策、不拼命令逻辑 |
| **TypeScript/Vue** | ~20% | 表单生成器、进度展示终端、状态管理、用户交互 | 不含业务逻辑、不含 ffmpeg 知识 |

### 设计原则

**Lua 主权**：Lua 是"总指挥"，Rust 是"执行代理"，前端是"展示终端"。

```
用户点击 [生成GIF]
→ 前端 invoke('run_preset', {preset, params, input, start, end, output})
→ Rust 加载 Lua 模块，调用 M.validate(params, input_info)
  → 若失败，直接返回错误给前端
→ Rust 调用 M.build_command_pipeline(...) 得到步骤数组
→ Rust 依次启动 ffmpeg，每收到一行 stderr 调用 M.parse_progress(...)
  → 进度事件推送给前端
→ 所有步骤完成后，调用 M.on_complete(output, params)
  → 执行后处理，返回消息给前端

全程 Rust 不知道步骤有几个、进度如何计算，完全交由 Lua 决策。
```

### 目录结构

```
src/
├── App.vue              # 主入口：状态机 (idle → validating → converting → done/error)
├── api.ts               # Tauri invoke 封装 (validateParams, executeConversion)
├── utils.ts             # 工具函数 (timeToSeconds, secondsToTime, isValidTimeRange)
└── components/
    ├── FileSelector.vue  # 文件选择 + 起止时间输入 + 时间范围校验
    ├── PresetSelector.vue # Lua 预设列表加载
    ├── ParamPanel.vue    # 动态参数面板 (从 Lua 预设解析控件)
    ├── ProgressView.vue  # 实时进度条 + 步骤日志
    └── ResultView.vue    # GIF 预览 + 打开输出文件夹

src-tauri/src/
├── lib.rs               # Tauri 命令注册入口
├── commands.rs          # scan_presets, get_controls, validate
├── commands_presets.rs  # list_presets, get_preset_controls
├── execute_commands.rs  # execute_conversion (完整管线执行)
├── ffmpeg_commands.rs   # run_ffmpeg, run_pipeline
├── ffmpeg_runner.rs     # FFmpeg 进程管理
├── lua_runtime.rs       # Lua 虚拟机：加载预设、调用函数
├── pipeline_executor.rs # 管线步骤执行 + 进度推送
└── types.rs             # 类型定义 (ConversionResult 等)

presets/
├── emoji_small.lua      # 小尺寸 emoji 动图预设 (两步：调色板 → GIF)
├── clear_demo.lua       # 清晰演示预设 (单步直接转 GIF)
└── custom_steps.lua     # 自定义步骤预设 (亮度对比度滤镜)
```

## 数据流

1. 选择视频文件 → 显示时长（后端提供时）
2. 选择 Lua 预设 → 动态加载控件定义
3. 调整参数 → 实时渲染对应控件
4. 点击「生成 GIF」→ 校验时间范围 + 参数合法性
5. 校验通过 → 调用 execute_conversion → 推送 conversion-progress 事件
6. 进度更新 → ProgressView 实时更新
7. 完成 → ResultView 展示 GIF + 打开文件夹

## 验收标准

### 单元测试
```bash
bash submit/test-agent-j.sh
```

### 端到端测试步骤
1. 启动应用 → 确认所有组件正常渲染
2. 选择视频文件 → 确认路径显示正确
3. 输入起止时间 → 确认结束时间 > 起始时间校验生效
4. 选择预设 → 确认参数面板动态加载对应控件
5. 点击「生成 GIF」→ 确认按钮进入禁用状态
6. 观察进度条 → 确认步骤名称和百分比实时更新
7. 等待完成 → 确认结果区域展示 GIF + 「打开文件夹」按钮
8. 错误场景 → 不选文件/预设时点击，确认错误提示展示

### 全链路检查清单
- [x] 所有核心组件存在 (FileSelector, PresetSelector, ParamPanel, ProgressView, ResultView)
- [x] 所有 Tauri commands 注册 (validate, execute_conversion, list_presets 等)
- [x] 所有 Lua 预设完整 (emoji_small, clear_demo, custom_steps)
- [x] 数据流串联 (ConversionResult, execute_conversion, conversion-progress)
- [x] 状态机管理 (idle → validating → converting → done/error)
- [x] 时间范围校验 (结束 > 起始)
- [x] 按钮禁用 (转换中不可重复点击)
- [x] 错误边界 (捕获错误并友好展示)
