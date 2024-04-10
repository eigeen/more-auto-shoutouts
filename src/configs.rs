use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use crate::game_context;

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("解析配置文件失败：{}", source))]
    Parse { source: toml::de::Error },
    #[snafu(display("读取配置文件失败：{}", source))]
    Io { source: std::io::Error },
    #[snafu(display("验证配置文件失败：{reason}"))]
    Validate { reason: String },
}

/// 配置文件
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// 全局事件冷却时间
    ///
    /// 默认应用于所有触发器
    ///
    /// 可被触发器设置覆盖
    #[serde(default = "default_event_cd")]
    pub trigger_cd: f32,
    #[serde(default)]
    pub trigger: Vec<Trigger>,
}

fn default_event_cd() -> f32 {
    0.5
}

/// 触发器
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trigger {
    /// 名称：可选，用于标记该触发器名称
    pub name: Option<String>,
    /// 行为模式：标记如何执行触发器定义的行为
    pub action_mode: Option<ActionMode>,
    /// 触发器行为
    pub action: Vec<Action>,
    /// 触发器触发条件：当设置的条件被触发时，执行触发器行为。有且仅有一个
    pub trigger_on: TriggerCondition,
    /// 触发器检查条件：可选，可多个，需要全部满足才能触发
    #[serde(default)]
    pub check: Vec<CheckCondition>,
    /// 冷却时间（秒）
    /// 覆盖全局设置
    pub cooldown: Option<f32>,
    /// 记录触发次数
    pub enable_cnt: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    /// 命令
    pub cmd: Command,
    /// 参数
    pub param: String,
}

/// 触发器条件
///
/// 决定触发器触发的条件
///
/// 此处条件均只会在对应条件发生变化时触发一次
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerCondition {
    LongswordLevelChanged {
        new: Option<ValueCmp>,
        old: Option<ValueCmp>,
    },
    WeaponType {
        value: ValueCmp,
    },
    QuestState {
        value: ValueCmp,
    },
    Fsm {
        new: Option<FsmConfig>,
        old: Option<FsmConfig>,
    },
    UseItem {
        item_id: ValueCmp,
    },
    InsectGlaiveLight {
        red: Option<NewOldValueCmp>,
        white: Option<NewOldValueCmp>,
        yellow: Option<NewOldValueCmp>,
    },
    ChargeBlade {
        sword_charge_timer: Option<NewOldValueCmp>,
        shield_charge_timer: Option<NewOldValueCmp>,
        power_axe_timer: Option<NewOldValueCmp>,
        phials: Option<NewOldValueCmp>,
        sword_power: Option<NewOldValueCmp>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewOldValueCmp {
    pub new: Option<ValueCmp>,
    pub old: Option<ValueCmp>,
}

/// 触发器检查条件
///
/// 检查条件会在触发器被触发时，检查是否满足要求
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckCondition {
    LongswordLevel {
        value: ValueCmp,
    },
    WeaponType {
        value: ValueCmp,
    },
    QuestState {
        value: ValueCmp,
    },
    Fsm {
        value: FsmConfig,
    },
    Damage {
        damage: ValueCmp,
        fsm: FsmConfig,
        timeout: Option<i32>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FsmConfig {
    pub target: ValueCmp,
    pub id: ValueCmp,
}

impl PartialEq<game_context::Fsm> for FsmConfig {
    fn eq(&self, other: &game_context::Fsm) -> bool {
        self.target == other.target && self.id == other.id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum ValueCmp {
    /// 等于某个整数
    EqInt(i32),
    /// 高级值比较
    Cmp {
        gt: Option<i32>,
        ge: Option<i32>,
        lt: Option<i32>,
        le: Option<i32>,
        ne: Option<i32>,
        r#in: Option<Vec<i32>>,
        nin: Option<Vec<i32>>,
    },
    /// 特殊定义值（通常由特定触发器定义）
    Special(String),
}

impl PartialEq<i32> for ValueCmp {
    fn eq(&self, other: &i32) -> bool {
        match self {
            ValueCmp::EqInt(val) => val == other,
            ValueCmp::Cmp {
                gt,
                ge,
                lt,
                le,
                ne,
                r#in,
                nin,
            } => {
                (gt.map_or(true, |v| other > &v))
                    && (ge.map_or(true, |v| other >= &v))
                    && (lt.map_or(true, |v| other < &v))
                    && (le.map_or(true, |v| other <= &v))
                    && (ne.map_or(true, |v| other != &v))
                    && (r#in.as_ref().map_or(true, |v| v.contains(other)))
                    && (nin.as_ref().map_or(true, |v| !v.contains(other)))
            }
            ValueCmp::Special(_) => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Command {
    /// 发送聊天消息
    SendChatMessage,
}

/// 触发器行为模式
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionMode {
    /// 顺序执行所有
    SequentialAll,
    /// 顺序执行下一个
    SequentialOne,
    /// 随机执行一个
    Random,
}

pub fn load_config<P>(path: P) -> Result<Config, ConfigError>
where
    P: AsRef<Path>,
{
    let s: String = fs::read_to_string(path).context(IoSnafu)?;
    let mut config: Config = toml::from_str(&s).context(ParseSnafu)?;
    // 预验证config
    if config.trigger_cd < 0.0 {
        return Err(ConfigError::Validate {
            reason: "event_cd 不能小于0.0".to_string(),
        });
    }
    for t in config.trigger.iter_mut() {
        // 为Trigger应用全局默认设置
        t.cooldown = Some(t.cooldown.unwrap_or(config.trigger_cd));
        // 检查触发器条件
        match &t.trigger_on {
            TriggerCondition::LongswordLevelChanged { new, old } => {
                if new.is_none() && old.is_none() {
                    return Err(ConfigError::Validate {
                        reason: "LongswordLevelChanged 参数不能都为空".to_string(),
                    });
                }
            }
            _ => {}
        };
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use mhw_toolkit::game_util::WeaponType;

    use super::*;

    const EXAMPLE_FILE_PATH: &str = "mas-config.example.toml";

    #[test]
    fn test_value_cmp_i32() {
        let a = 10;
        let b = ValueCmp::EqInt(10);
        assert!(b == a);
    }

    #[test]
    fn test_value_cmp_i32_complex() {
        let a = 10;
        let b = ValueCmp::Cmp {
            gt: Some(5),
            ge: None,
            lt: None,
            le: None,
            ne: None,
            r#in: None,
            nin: None,
        };
        assert!(b == a);

        let a = 10;
        let b = ValueCmp::Cmp {
            gt: None,
            ge: None,
            lt: Some(5),
            le: None,
            ne: None,
            r#in: None,
            nin: None,
        };
        assert!(b != a);
    }

    #[test]
    fn test_value_cmp_f32_complex() {
        let a: f32 = 90.0;
        let b = ValueCmp::Cmp {
            gt: Some(0),
            ge: None,
            lt: None,
            le: None,
            ne: None,
            r#in: None,
            nin: None,
        };
        assert!(b == a as i32);
    }

    #[test]
    fn test_value_cmp_in_nin() {
        let a: f32 = 90.0;
        let b = ValueCmp::Cmp {
            gt: None,
            ge: None,
            lt: None,
            le: None,
            ne: None,
            r#in: Some(vec![10, 50, 90]),
            nin: None,
        };
        assert!(b == a as i32);

        let a: i32 = 20;
        let b = ValueCmp::Cmp {
            gt: None,
            ge: None,
            lt: None,
            le: None,
            ne: None,
            r#in: Some(vec![10, 50, 90]),
            nin: None,
        };
        assert!(b != a);
    }

    #[test]
    fn test_load_config() {
        let cfg = load_config(EXAMPLE_FILE_PATH).unwrap();
        eprintln!("{:?}", cfg);
    }

    #[test]
    fn test_valuecmp_weapon() {
        let longsword = WeaponType::LongSword;
        assert!(ValueCmp::EqInt(3) == longsword.as_i32());
    }

    #[test]
    fn test_convert_to_json() {
        let cfg = load_config(EXAMPLE_FILE_PATH).unwrap();
        let json_cfg = serde_json::to_string(&cfg).unwrap();
        eprintln!("{}", json_cfg);
    }
}
