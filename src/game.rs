use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{DateTime, TimeDelta, Utc};
use log::{debug, info};
use mhw_toolkit::{
    game_util::{self, WeaponType},
    util,
};
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, Notify};

use crate::game_context::{ChargeBlade, ChatCommand, Fsm, InsectGlaive, SpecializedTool};

const QUEST_BASE: *const i32 = 0x14500CAF0 as *const i32;
const QUEST_OFFSETS: isize = 0x38;
const PLAYER_BASE: *const i32 = 0x145011760 as *const i32;
const PLAYER_FSMTARGET_OFFSETS: &[isize] = &[0x50, 0x6274];
const PLAYER_FSMID_OFFSETS: &[isize] = &[0x50, 0x6278];
const USE_ITEM_OFFSETS: &[isize] = &[0x50, 0x80, 0x80, 0x10, 0x288, 0x28E0];

const WEAPON_DATA_BASE: *const i32 = 0x145011760 as *const i32;
const LONGSWORD_OFFSETS: &[isize] = &[0x50, 0x468, 0x70, 0x10, 0x18, 0x2370];
const WEAPON_OFFSETS: &[isize] = &[0x50, 0xC0, 0x8, 0x78, 0x2E8];
const WEAPON_DATA_OFFSETS: &[isize] = &[0x50, 0x76B0];

const CHARGE_BLADE_BASE: *const i32 = 0x1450EA510 as *const i32;
const CHARGE_BLADE_MAX_PHIALS_OFFSETS: &[isize] = &[0x110, 0x98, 0x58, 0x5F98];

const CHAT_COMMAND_PREFIX: &str = "!mas ";
static CHAT_MESSAGE_RECV: Lazy<game_util::ChatMessageReceiver> = Lazy::new(|| {
    let mut instance = game_util::ChatMessageReceiver::new();
    instance.set_prefix_filter(CHAT_COMMAND_PREFIX);
    instance
});
static DAMAGE_COLLECTOR: Lazy<Arc<DamageCollector>> = Lazy::new(|| Arc::new(DamageCollector::new()));

pub struct DamageCollector {
    records: Mutex<HashMap<Fsm, Vec<DamageData>>>,
    fsm_notify: Notify,
    simple_collector: Mutex<Vec<DamageData>>,
    now_fsm: Mutex<Fsm>,
}

impl DamageCollector {
    fn new() -> Self {
        DamageCollector {
            records: Mutex::new(HashMap::new()),
            fsm_notify: Notify::new(),
            simple_collector: Mutex::new(Vec::new()),
            now_fsm: Mutex::new(Fsm::default()),
        }
    }

    /// 接收伤害事件
    pub async fn on_damage(&self, damage: i32) {
        let fsm = get_fsm();
        debug!("DamageCollector: on damage {} <=> {:?}", damage, fsm);
        // 记录伤害
        let data = DamageData::new(damage, &fsm);
        self.records.lock().await.entry(fsm).or_insert_with(Vec::new).push(data.clone());
        self.simple_collector.lock().await.push(data);
    }

    /// 接收fsm变更事件
    pub async fn on_fsm_changed(&self, fsm_after: &Fsm) {
        self.records.lock().await.remove(fsm_after);
        *self.now_fsm.lock().await = *fsm_after;
        self.clear_expired_data().await;
        self.fsm_notify.notify_waiters();
    }

    /// 获取伤害收集器实例
    pub fn instance() -> Arc<DamageCollector> {
        DAMAGE_COLLECTOR.clone()
    }

    /// 获取某个fsm的收集值
    async fn collect_one(&self, fsm: &Fsm) -> i32 {
        if let Some(record) = self.records.lock().await.get(fsm) {
            record.iter().map(|data| data.damage).sum()
        } else {
            0
        }
    }

    /// 收集某个时间段的伤害
    async fn _collect_duration(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> i32 {
        let mut result_damage = 0;
        let simple_collector = self.simple_collector.lock().await;
        // 使用二分搜索确定开始时间的索引
        let start_idx = match simple_collector.binary_search_by(|data| data.time.cmp(&start_time)) {
            Ok(index) => index,
            Err(index) => index,
        };

        // 遍历从开始时间到结束时间的数据
        for data in simple_collector[start_idx..].iter() {
            if data.time > end_time {
                break; // 超出结束时间，停止遍历
            }
            result_damage += data.damage;
        }

        result_damage
    }

    /// 清除过期的数据
    async fn clear_expired_data(&self) {
        let now = Utc::now();
        let mut simple_collector = self.simple_collector.lock().await;
        let mut cut_index = 0;
        for (idx, data) in simple_collector.iter().enumerate() {
            // 清除60秒之前的数据
            if data.time + TimeDelta::try_minutes(60).unwrap() < now {
                cut_index = idx;
                break;
            }
        }
        simple_collector.drain(0..cut_index);
    }

    /// 收集某个fsm期间的伤害
    pub async fn collect_fsm(&self, fsm: &Fsm, timeout_dur: Duration) -> i32 {
        match tokio::time::timeout(timeout_dur, async {
            loop {
                // fsm变化通知
                self.fsm_notify.notified().await;
                if *self.now_fsm.lock().await != *fsm {
                    return self.collect_one(fsm).await;
                }
            }
        })
        .await
        {
            Ok(damage) => damage,
            Err(_) => {
                // 超时返回当前收集值
                self.collect_one(fsm).await
            }
        }
    }

    /// 收集从现在开始一段时间内的伤害
    pub async fn collect_time(&self, duration: Duration) -> i32 {
        let start_time = Utc::now();
        let end_time = start_time + duration;
        tokio::time::sleep(duration).await;
        // 收集伤害
        self._collect_duration(start_time, end_time).await
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct DamageData {
    damage: i32,
    fsm: Fsm,
    time: DateTime<Utc>,
}

impl DamageData {
    pub fn new(damage: i32, fsm: &Fsm) -> Self {
        DamageData {
            damage,
            fsm: *fsm,
            time: Utc::now(),
        }
    }
}

pub fn get_chat_command() -> Option<ChatCommand> {
    if let Some(msg) = CHAT_MESSAGE_RECV.try_recv() {
        debug!("接收用户命令消息：{}", msg);
        let cmd = ChatCommand::from_str(&msg[CHAT_COMMAND_PREFIX.len()..]);
        if cmd.is_none() {
            info!("无效的命令：{}", msg);
            game_util::send_chat_message("无效的命令");
        }
        cmd
    } else {
        None
    }
}

pub fn get_quest_state() -> i32 {
    util::get_value_with_offset(QUEST_BASE, &[QUEST_OFFSETS]).unwrap_or(0)
}

pub fn get_longsword_level() -> i32 {
    util::get_value_with_offset(WEAPON_DATA_BASE, LONGSWORD_OFFSETS).unwrap_or(99)
}

pub fn get_weapon_type() -> WeaponType {
    let weapon_type_id = util::get_value_with_offset(WEAPON_DATA_BASE, WEAPON_OFFSETS).unwrap_or(0);
    WeaponType::from_i32(weapon_type_id)
}

pub fn get_fsm() -> Fsm {
    let id = util::get_value_with_offset(PLAYER_BASE, PLAYER_FSMID_OFFSETS).unwrap_or(0);
    let target = util::get_value_with_offset(PLAYER_BASE, PLAYER_FSMTARGET_OFFSETS).unwrap_or(0);
    Fsm { id, target }
}

pub fn get_use_item_id() -> i32 {
    util::get_value_with_offset(PLAYER_BASE, USE_ITEM_OFFSETS).unwrap_or(-1)
}

pub fn get_insect_glaive_data() -> Option<InsectGlaive> {
    let weapon_data_base = match util::get_ptr_with_offset(WEAPON_DATA_BASE as *const f32, WEAPON_DATA_OFFSETS) {
        Some(ptr) => ptr,
        None => return None,
    };
    let data: InsectGlaive = InsectGlaive {
        attack_timer: util::get_value_with_offset(weapon_data_base, &[0x2368]).unwrap_or(0.0),
        speed_timer: util::get_value_with_offset(weapon_data_base, &[0x236C]).unwrap_or(0.0),
        defense_timer: util::get_value_with_offset(weapon_data_base, &[0x2370]).unwrap_or(0.0),
    };

    Some(data)
}

pub fn get_charge_blade_data() -> Option<ChargeBlade> {
    let weapon_data_base = match util::get_ptr_with_offset(WEAPON_DATA_BASE as *const f32, WEAPON_DATA_OFFSETS) {
        Some(ptr) => ptr,
        None => return None,
    };
    let data: ChargeBlade = ChargeBlade {
        sword_power: util::get_value_with_offset(weapon_data_base, &[0x2370]).unwrap_or(0.0),
        sword_charge_timer: util::get_value_with_offset(weapon_data_base, &[0x237C]).unwrap_or(0.0),
        shield_charge_timer: util::get_value_with_offset(weapon_data_base, &[0x2378]).unwrap_or(0.0),
        phials: util::get_value_with_offset(weapon_data_base as *const i32, &[0x2374]).unwrap_or(0),
        max_phials: util::get_value_with_offset(CHARGE_BLADE_BASE, CHARGE_BLADE_MAX_PHIALS_OFFSETS).unwrap_or(0),
        power_axe_mode: util::get_value_with_offset(weapon_data_base as *const i32, &[0x2474]).unwrap_or(0),
        power_axe_timer: util::get_value_with_offset(weapon_data_base, &[0x2470]).unwrap_or(0.0),
    };

    Some(data)
}

// TODO
pub fn get_specialized_tool() -> Option<SpecializedTool> {
    None
}
