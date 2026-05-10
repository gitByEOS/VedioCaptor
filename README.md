# VideoCaptor (agent-j)

Lua主控视频转GIF工具：Rust核心 + Tauri前端 + Lua预设脚本。Lua负责流程编排、参数校验、命令构建、进度解析、后处理，Rust仅做执行代理。

## 项目架构

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
├── emoji_small.lua      # 小尺寸 emoji 动图预设
├── clear_demo.lua       # 清晰演示预设
└── custom_steps.lua     # 自定义步骤预设
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
