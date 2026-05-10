use mlua::{Lua, Table, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::pipeline_executor::Step;
use crate::types::{ControlDef, ProgressEvent, ValidateResult, VideoInfo};

/// Lua 运行时代理
pub struct LuaRuntime {
    lua: Lua,
}

impl LuaRuntime {
    pub fn new() -> Self {
        Self { lua: Lua::new() }
    }

    /// 加载 .lua 预设文件并执行
    pub fn load_preset(path: &str) -> Result<Self, String> {
        let source = fs::read_to_string(path)
            .map_err(|e| format!("无法读取预设文件 {}: {}", path, e))?;

        let runtime = Self::new();
        runtime
            .lua
            .load(&source)
            .exec()
            .map_err(|e| format!("Lua 执行错误: {}", e))?;

        Ok(runtime)
    }

    /// 从源码创建运行时（用于内部测试）
    #[allow(dead_code)]
    pub fn from_source(source: &str) -> Result<Self, String> {
        let runtime = Self::new();
        runtime
            .lua
            .load(source)
            .exec()
            .map_err(|e| format!("Lua 执行错误: {}", e))?;
        Ok(runtime)
    }

    /// 扫描目录下所有 .lua 文件，返回预设名列表
    pub fn scan_presets(dir: &str) -> Vec<String> {
        let dir_path = Path::new(dir);
        if !dir_path.is_dir() {
            log::warn("预设目录不存在", dir);
            return vec![];
        }

        match fs::read_dir(dir_path) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext == "lua")
                })
                .filter_map(|entry| {
                    entry
                        .path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(String::from)
                })
                .collect(),
            Err(e) => {
                log::warn("无法读取预设目录", &e.to_string());
                vec![]
            }
        }
    }

    /// 调用 Lua 的 get_controls 函数，返回控件定义
    pub fn get_controls(&self) -> Result<Vec<ControlDef>, String> {
        let func: mlua::Function = self
            .lua
            .globals()
            .get("get_controls")
            .map_err(|_| "Lua 中未定义 get_controls 函数".to_string())?;

        let result: Table = func
            .call(())
            .map_err(|e| format!("调用 get_controls 失败: {}", e))?;

        parse_controls_table(&result)
    }

    /// 调用 Lua 的 validate 函数校验参数
    pub fn validate(
        &self,
        params: &HashMap<String, serde_json::Value>,
        input_info: &VideoInfo,
    ) -> Result<ValidateResult, String> {
        let func: mlua::Function = self
            .lua
            .globals()
            .get("validate")
            .map_err(|_| "Lua 中未定义 validate 函数".to_string())?;

        let params_table = self.build_params_table(params);
        let info_table = self.build_video_info_table(input_info);

        let result: Table = func
            .call((params_table, info_table))
            .map_err(|e| format!("调用 validate 失败: {}", e))?;

        parse_validate_result(&result)
    }

    /// 调用 Lua 的 build_command_pipeline，返回可执行步骤
    pub fn build_command_pipeline(
        &self,
        params: &HashMap<String, serde_json::Value>,
        input_path: &str,
        output_path: &str,
    ) -> Result<Vec<Step>, String> {
        let func: mlua::Function = self
            .lua
            .globals()
            .get("build_command_pipeline")
            .map_err(|_| "Lua 中未定义 build_command_pipeline 函数".to_string())?;

        let params_table = self.build_params_table(params);
        let input_str = self.lua.create_string(input_path).map_err(|e| format!("创建输入路径字符串失败: {}", e))?;
        let output_str = self.lua.create_string(output_path).map_err(|e| format!("创建输出路径字符串失败: {}", e))?;

        let result: Table = func
            .call((params_table, input_str, output_str))
            .map_err(|e| format!("调用 build_command_pipeline 失败: {}", e))?;

        parse_pipeline_steps(&result)
    }

    /// 调用 Lua 的 on_complete 获取后处理消息
    pub fn on_complete(&self, output_path: &str, params: &HashMap<String, serde_json::Value>) -> Result<Option<String>, String> {
        let func: Result<mlua::Function, _> = self.lua.globals().get("on_complete");
        let func = match func {
            Ok(f) => f,
            Err(_) => return Ok(None),
        };

        let params_table = self.build_params_table(params);
        let output_str = self.lua.create_string(output_path).map_err(|e| format!("创建输出路径字符串失败: {}", e))?;

        let result: mlua::Value = func
            .call((output_str, params_table))
            .map_err(|e| format!("调用 on_complete 失败: {}", e))?;

        match result {
            Value::String(s) => {
                let msg = s.to_str().map(|b| b.to_string()).unwrap_or_default();
                if msg.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(msg))
                }
            }
            Value::Nil => Ok(None),
            _ => Ok(None),
        }
    }

    /// 调用 Lua 的 parse_progress 解析 ffmpeg stderr 行
    /// 若 Lua 中未定义该函数或解析失败，返回默认进度 0
    pub fn parse_progress(&self, line: &str, step_index: usize, step_name: &str) -> ProgressEvent {
        let func: Result<mlua::Function, _> = self.lua.globals().get("parse_progress");
        match func {
            Ok(func) => {
                let args = vec![
                    mlua::Value::String(self.lua.create_string(line).expect("创建 Lua 字符串失败")),
                    mlua::Value::Integer(step_index as i64),
                    mlua::Value::String(self.lua.create_string(step_name).expect("创建 Lua 字符串失败")),
                ];
                let args = mlua::MultiValue::from_vec(args);
                match func.call::<mlua::Value>(args) {
                    Ok(mlua::Value::Table(table)) => {
                        let progress: f64 = table.get("progress").unwrap_or(0.0);
                        let message: String = table.get("message").unwrap_or_default();
                        ProgressEvent {
                            step_name: step_name.to_string(),
                            step_index,
                            total_steps: 0,
                            progress,
                            message,
                        }
                    }
                    _ => ProgressEvent {
                        step_name: step_name.to_string(),
                        step_index,
                        total_steps: 0,
                        progress: 0.0,
                        message: line.to_string(),
                    },
                }
            }
            Err(_) => ProgressEvent {
                step_name: step_name.to_string(),
                step_index,
                total_steps: 0,
                progress: 0.0,
                message: String::new(),
            },
        }
    }

    /// 在当前 Lua 实例中构建参数表
    fn build_params_table(&self, params: &HashMap<String, serde_json::Value>) -> Table {
        let table = self
            .lua
            .create_table()
            .expect("创建 Lua 表失败");
        for (key, value) in params {
            let lua_value = json_to_lua_value(value, &self.lua);
            let _ = table.set(key.as_str(), lua_value);
        }
        table
    }

    /// 在当前 Lua 实例中构建视频信息表
    fn build_video_info_table(&self, info: &VideoInfo) -> Table {
        let table = self
            .lua
            .create_table()
            .expect("创建 Lua 表失败");
        let _ = table.set("duration", info.duration);
        let _ = table.set("width", info.width as i64);
        let _ = table.set("height", info.height as i64);
        table
    }
}

/// 将 JSON Value 转换为 mlua Value
fn json_to_lua_value(value: &serde_json::Value, lua: &Lua) -> Value {
    match value {
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Nil
            }
        }
        serde_json::Value::String(s) => Value::String(lua.create_string(s).unwrap()),
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Array(arr) => {
            let table = lua.create_table().unwrap();
            for (i, item) in arr.iter().enumerate() {
                let _ = table.set(i + 1, json_to_lua_value(item, lua));
            }
            Value::Table(table)
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table().unwrap();
            for (k, v) in obj {
                let _ = table.set(k.as_str(), json_to_lua_value(v, lua));
            }
            Value::Table(table)
        }
    }
}

/// 将 Lua 控件表解析为 ControlDef 列表
fn parse_controls_table(table: &Table) -> Result<Vec<ControlDef>, String> {
    let mut controls = Vec::new();

    for pair in table.pairs::<mlua::Value, mlua::Value>() {
        let (_, value) = pair.map_err(|e| format!("解析控件失败: {}", e))?;
        if let Value::Table(ctrl) = value {
            if let Ok(def) = parse_single_control(&ctrl) {
                controls.push(def);
            }
        }
    }

    Ok(controls)
}

/// 解析单个控件定义
fn parse_single_control(table: &Table) -> Result<ControlDef, String> {
    let ctrl_type: String = table.get("type").unwrap_or_else(|_| String::from("number"));
    let label: String = table.get("label").unwrap_or_default();

    match ctrl_type.as_str() {
        "slider" => Ok(ControlDef::Slider {
            label,
            min: table.get("min").unwrap_or(0.0),
            max: table.get("max").unwrap_or(100.0),
            default: table.get("default").unwrap_or(50.0),
        }),
        "select" => {
            let values = extract_string_array(table, "values");
            let default: String = table.get("default").unwrap_or_default();
            Ok(ControlDef::Select {
                label,
                values,
                default,
            })
        }
        _ => Ok(ControlDef::Number {
            label,
            min: table.get("min").unwrap_or(0.0),
            max: table.get("max").unwrap_or(100.0),
            default: table.get("default").unwrap_or(0.0),
        }),
    }
}

/// 从 Lua 表中提取字符串数组
fn extract_string_array(table: &Table, key: &str) -> Vec<String> {
    let inner: Result<Table, _> = table.get(key);
    inner
        .ok()
        .map(|t| {
            t.sequence_values()
                .filter_map(|v: mlua::Result<mlua::Value>| v.ok())
                .filter_map(|v| {
                    if let Value::String(s) = v {
                        let borrowed = s.to_str().ok()?;
                        Some(borrowed.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 解析 Lua 校验结果表
fn parse_validate_result(table: &Table) -> Result<ValidateResult, String> {
    let ok: bool = table.get("ok").unwrap_or(false);
    let error: Option<String> = table.get("error").ok();
    Ok(ValidateResult { ok, error })
}

/// 解析 Lua 返回的步骤数组
fn parse_pipeline_steps(table: &Table) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();

    for pair in table.sequence_values::<mlua::Value>() {
        let value = pair.map_err(|e| format!("解析步骤失败: {}", e))?;
        if let Value::Table(step_table) = value {
            let desc: String = step_table.get("desc").unwrap_or_else(|_| String::from("未命名步骤"));
            let args_table: Table = step_table
                .get("args")
                .map_err(|_| "步骤缺少 args 字段".to_string())?;
            let command = build_command_from_args(&args_table)?;
            steps.push(Step {
                step_name: desc,
                command,
            });
        }
    }

    Ok(steps)
}

/// 将 Lua args 数组拼接为可执行命令行字符串
fn build_command_from_args(args_table: &Table) -> Result<String, String> {
    let mut parts = Vec::new();
    let mut i = 1;
    loop {
        let val: Result<Value, _> = args_table.get(i);
        match val {
            Ok(Value::String(s)) => {
                let borrowed = s.to_str().map(|b| b.to_string()).unwrap_or_default();
                parts.push(quote_if_needed(&borrowed));
            }
            Ok(_) => break,
            Err(_) => break,
        }
        i += 1;
    }
    if parts.is_empty() {
        return Err("args 数组为空".to_string());
    }

    // 如果第一个参数不是 ffmpeg/ffprobe，添加 ffmpeg 前缀
    if parts[0] != "ffmpeg" && parts[0] != "ffprobe" {
        parts.insert(0, "ffmpeg".to_string());
    }

    Ok(parts.join(" "))
}

/// 包含空格或特殊字符的字符串加引号
fn quote_if_needed(s: &str) -> String {
    if s.contains(' ') || s.contains('"') || s.contains('\'') || s.is_empty() {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// 日志模块
mod log {
    pub fn warn(msg: &str, detail: &str) {
        eprintln!("[WARN] {}: {}", msg, detail);
    }
}
