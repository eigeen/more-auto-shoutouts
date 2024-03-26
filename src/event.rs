use std::thread;

use log::error;
use mhw_toolkit::game_util::WeaponType;

use crate::{
    game_context::{Context, Fsm},
    triggers::TriggerManager,
};

/// 事件
#[derive(Debug)]
pub enum Event {
    LoadTriggers {
        trigger_mgr: TriggerManager,
    },
    LongswordLevelChanged {
        new: i32,
        old: i32,
        ctx: Context,
    },
    WeaponTypeChanged {
        new: WeaponType,
        old: WeaponType,
        ctx: Context,
    },
    QuestStateChanged {
        new: i32,
        old: i32,
        ctx: Context,
    },
    FsmChanged {
        new: Fsm,
        old: Fsm,
        ctx: Context,
    },
    UseItem {
        item_id: i32,
        ctx: Context,
    },
    InsectGlaive {
        ctx: Context,
    },
    ChargeBlade {
        ctx: Context,
    },
    Damage {
        damage: i32,
    },
}

impl Event {
    pub fn extract_ctx(&self) -> Context {
        match self {
            Event::LoadTriggers { .. } => Self::panic_if_no_ctx("Event::LoadTriggers"),
            Event::LongswordLevelChanged { ctx, .. } => ctx.clone(),
            Event::WeaponTypeChanged { ctx, .. } => ctx.clone(),
            Event::QuestStateChanged { ctx, .. } => ctx.clone(),
            Event::FsmChanged { ctx, .. } => ctx.clone(),
            Event::InsectGlaive { ctx } => ctx.clone(),
            Event::ChargeBlade { ctx } => ctx.clone(),
            Event::UseItem { ctx, .. } => ctx.clone(),
            Event::Damage { .. } => Self::panic_if_no_ctx("Event::Damage"),
        }
    }

    pub fn event_type(&self) -> EventType {
        match self {
            Event::LoadTriggers { .. } => EventType::LoadTriggers,
            Event::LongswordLevelChanged { .. } => EventType::LongswordLevelChanged,
            Event::WeaponTypeChanged { .. } => EventType::WeaponTypeChanged,
            Event::QuestStateChanged { .. } => EventType::QuestStateChanged,
            Event::FsmChanged { .. } => EventType::FsmChanged,
            Event::InsectGlaive { .. } => EventType::InsectGlaive,
            Event::ChargeBlade { .. } => EventType::ChargeBlade,
            Event::UseItem { .. } => EventType::UseItem,
            Event::Damage { .. } => EventType::Damage,
        }
    }

    fn panic_if_no_ctx(target: &str) -> ! {
        error!("trying to get context from {target}, panicked");
        // 只是防止没打出日志退出，可能大概有用吧
        thread::sleep(std::time::Duration::from_millis(500));
        panic!("trying to get context from {target}, panicked")
    }
}

/// 不携带数据的事件类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    LoadTriggers,
    LongswordLevelChanged,
    WeaponTypeChanged,
    QuestStateChanged,
    FsmChanged,
    UseItem,
    InsectGlaive,
    ChargeBlade,
    Damage,
}
