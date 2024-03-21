use crate::game;
use mhw_toolkit::game_util::WeaponType;

/// 游戏内状态记录上下文
///
/// 记录当前游戏内某些值
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub chat_command: Option<ChatCommand>,
    pub quest_state: i32,
    pub longsword_level: i32,
    pub weapon_type: WeaponType,
    pub fsm: Fsm,
    pub use_item_id: i32,
    pub insect_glaive: InsectGlaive,
    pub charge_blade: ChargeBlade,
    pub specialized_tool: Option<SpecializedTool>,

    pub last_ctx: Option<Box<Context>>,
}

impl Context {
    pub fn update_context(&mut self) {
        self.chat_command = game::get_chat_command();
        self.quest_state = game::get_quest_state();
        self.weapon_type = game::get_weapon_type();
        self.fsm = game::get_fsm();
        self.use_item_id = game::get_use_item_id();
        self.longsword_level = if WeaponType::LongSword == self.weapon_type {
            game::get_longsword_level()
        } else {
            0
        };
        self.insect_glaive = if WeaponType::InsectGlaive == self.weapon_type {
            game::get_insect_glaive_data().unwrap_or_default()
        } else {
            InsectGlaive::default()
        };
        self.charge_blade = if WeaponType::ChargeBlade == self.weapon_type {
            game::get_charge_blade_data().unwrap_or_default()
        } else {
            ChargeBlade::default()
        };
        self.specialized_tool = game::get_specialized_tool();
    }

    pub fn store_last_context(&mut self) {
        self.last_ctx = None;
        self.last_ctx = Some(Box::new(self.clone()));
    }
}

/// 动作
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fsm {
    pub target: i32,
    pub id: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChatCommand {
    ReloadConfig,
}

impl ChatCommand {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "reload" => Some(ChatCommand::ReloadConfig),
            _ => None,
        }
    }
}

/// 操虫棍
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InsectGlaive {
    /// 红灯时间
    pub attack_timer: f32,
    /// 白灯时间
    pub speed_timer: f32,
    /// 黄灯时间
    pub defense_timer: f32,
}

/// 盾斧
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChargeBlade {
    /// 剑能量
    pub sword_power: f32,
    /// 红剑时间
    pub sword_charge_timer: f32,
    /// 红盾时间
    pub shield_charge_timer: f32,
    /// 瓶子数量
    pub phials: i32,
    /// 最大瓶子数量
    pub max_phials: i32,
    /// 斧强化（电锯）模式
    /// 0 常规
    /// 255 电锯
    pub power_axe_mode: i32,
    /// 斧强化（电锯）时间（单个瓶子）
    pub power_axe_timer: f32,
}

#[allow(dead_code)]
impl ChargeBlade {
    pub fn is_power_axe(&self) -> bool {
        self.power_axe_mode == 255
    }
}

/// 特殊装备
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpecializedTool {}
