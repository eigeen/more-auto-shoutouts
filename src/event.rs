use mhw_toolkit::game_util::WeaponType;

use crate::{
    game_context::{Context, Fsm},
    triggers::TriggerManager,
};

/// 事件
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Event {
    LoadTriggers { trigger_mgr: TriggerManager },
    UpdateContext { ctx: Context },
    LongswordLevelChanged { new: i32, old: i32 },
    WeaponTypeChanged { new: WeaponType, old: WeaponType },
    QuestStateChanged { new: i32, old: i32 },
    FsmChanged { new: Fsm, old: Fsm },
    UseItem { item_id: i32 },
    InsectGlaive,
    ChargeBlade,
    Damage { damage: i32 },
}

impl Event {
    pub fn event_type(&self) -> EventType {
        match self {
            Event::LoadTriggers { .. } => EventType::LoadTriggers,
            Event::UpdateContext { .. } => EventType::UpdateContext,
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
}

/// 不携带数据的事件类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    LoadTriggers,
    UpdateContext,
    LongswordLevelChanged,
    WeaponTypeChanged,
    QuestStateChanged,
    FsmChanged,
    UseItem,
    InsectGlaive,
    ChargeBlade,
    Damage,
}
