use serde::{Deserialize, Serialize};

/// 控件类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlDef {
    Slider {
        label: String,
        min: f64,
        max: f64,
        default: f64,
    },
    Select {
        label: String,
        values: Vec<String>,
        default: String,
    },
    Number {
        label: String,
        min: f64,
        max: f64,
        default: f64,
    },
}

/// 视频输入信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
}

/// 参数校验结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResult {
    pub ok: bool,
    pub error: Option<String>,
}

/// 预设模块
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PresetModule {
    pub name: String,
    pub lua_source: String,
}
