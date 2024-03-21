use std::sync::atomic::{AtomicI32, Ordering};

use log::error;
use mhw_toolkit::game_util;
use once_cell::sync::Lazy;
use rand::Rng;

use crate::{
    conditions::{
        charge_blade::ChargeBladeCondition, fsmid::FsmIDCondition, insect_glaive::InsectGlaiveCondition,
        longsword::LongswordCondition, quest_state::QuestStateCondition, use_item::UseItemCondition,
        weapon_id::WeaponTypeCondition,
    },
    configs::{self, ActionMode, TriggerCondition},
    game_context::{Context, Fsm},
};

static CHAT_MESSAGE_SENDER: Lazy<game_util::ChatMessageSender> = Lazy::new(|| game_util::ChatMessageSender::new());

#[derive(Debug)]
pub enum Event {
    LoadTriggers { trigger_mgr: TriggerManager },
    LongswordLevelChanged { new: i32, old: i32, ctx: Context },
    WeaponTypeChanged { new: i32, old: i32, ctx: Context },
    QuestStateChanged { new: i32, old: i32, ctx: Context },
    FsmChanged { new: Fsm, old: Fsm, ctx: Context },
    UseItem { item_id: i32, ctx: Context },
    InsectGlaive { ctx: Context },
    ChargeBlade { ctx: Context },
}

impl Event {
    pub fn extract_ctx(&self) -> Context {
        match self {
            Event::LoadTriggers { .. } => {
                error!("trying to get context from Event::LoadTriggers, panicked");
                panic!("trying to get context from Event::LoadTriggers, panicked")
            }
            Event::LongswordLevelChanged { ctx, .. } => ctx.clone(),
            Event::WeaponTypeChanged { ctx, .. } => ctx.clone(),
            Event::QuestStateChanged { ctx, .. } => ctx.clone(),
            Event::FsmChanged { ctx, .. } => ctx.clone(),
            Event::InsectGlaive { ctx } => ctx.clone(),
            Event::ChargeBlade { ctx } => ctx.clone(),
            Event::UseItem { ctx, .. } => ctx.clone(),
        }
    }
}

pub trait AsTriggerCondition: Send {
    fn check(&self, event: &Event) -> bool;
}

pub trait AsCheckCondition: Send {
    fn check(&self, context: &Context) -> bool;
}

pub trait AsAction: Send {
    fn execute(&self);
}

pub struct Trigger {
    actions: Vec<Box<dyn AsAction>>,
    trigger_condition: Box<dyn AsTriggerCondition>,
    check_conditions: Vec<Box<dyn AsCheckCondition>>,
    event_mode: ActionMode,
    event_idx: AtomicI32,
}

impl Trigger {
    pub fn new(event_mode: ActionMode, trigger_condition: Box<dyn AsTriggerCondition>) -> Trigger {
        Trigger {
            actions: Vec::new(),
            trigger_condition,
            check_conditions: Vec::new(),
            event_mode,
            event_idx: AtomicI32::new(0),
        }
    }

    pub fn add_action(&mut self, event: Box<dyn AsAction>) {
        self.actions.push(event)
    }

    pub fn add_check_condition(&mut self, cond: Box<dyn AsCheckCondition>) {
        self.check_conditions.push(cond)
    }

    fn execute_next_event(&self) {
        let event_idx = self.event_idx.fetch_add(1, Ordering::SeqCst);
        if event_idx >= self.actions.len() as i32 {
            self.event_idx.store(0, Ordering::SeqCst);
        }
        self.actions[event_idx as usize].execute();
    }

    fn execute_random_one(&self) {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.actions.len());
        self.actions[idx].execute();
    }

    pub fn process(&self, event: &Event) {
        // 判断触发器
        if !self.trigger_condition.check(event) {
            return;
        }
        // 判断检查器
        let checked = self.check_conditions.iter().all(|c| c.check(&event.extract_ctx()));
        if !checked {
            return;
        }
        match self.event_mode {
            ActionMode::SequentialAll => self.actions.iter().for_each(|e| {
                e.execute();
            }),
            ActionMode::SequentialOne => {
                self.execute_next_event();
            }
            ActionMode::Random => {
                self.execute_random_one();
            }
        }
    }
}

pub struct SendChatMessageEvent {
    msg: String,
}

impl SendChatMessageEvent {
    pub fn new(msg: &str) -> Self {
        SendChatMessageEvent { msg: msg.to_string() }
    }
}

impl AsAction for SendChatMessageEvent {
    fn execute(&self) {
        CHAT_MESSAGE_SENDER.send(&self.msg);
    }
}

/// 触发器管理
pub struct TriggerManager {
    triggers: Vec<Trigger>,
}

impl std::fmt::Debug for TriggerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TriggerManager").field("triggers.len", &self.triggers.len()).finish()
    }
}

impl TriggerManager {
    pub fn new() -> Self {
        TriggerManager { triggers: Vec::new() }
    }

    pub fn register_trigger(&mut self, trigger: Trigger) {
        self.triggers.push(trigger);
    }

    pub fn process_all(&self, event: &Event) {
        self.triggers.iter().for_each(|t| {
            t.process(event);
        });
    }
}

pub fn parse_trigger(t_cfg: &configs::Trigger) -> Trigger {
    let t_cfg = t_cfg.clone();
    let event_mode = t_cfg.action_mode.unwrap_or(configs::ActionMode::SequentialAll);
    let t_cond = parse_trigger_condition(&t_cfg.trigger_on);

    let mut t = Trigger::new(event_mode, t_cond);
    t_cfg.check.iter().map(parse_check_condition).for_each(|c| t.add_check_condition(c));
    t_cfg.action.iter().filter_map(parse_event).for_each(|e| t.add_action(e));

    log::debug!("注册trigger check({}), action({})", t.check_conditions.len(), t.actions.len());

    t
}

fn parse_check_condition(check_cond: &configs::CheckCondition) -> Box<dyn AsCheckCondition> {
    match check_cond {
        configs::CheckCondition::LongswordLevel { .. } => Box::new(LongswordCondition::new_check(&check_cond)),
        configs::CheckCondition::WeaponType { .. } => Box::new(WeaponTypeCondition::new_check(&check_cond)),
        configs::CheckCondition::QuestState { .. } => Box::new(QuestStateCondition::new_check(&check_cond)),
        configs::CheckCondition::Fsm { .. } => Box::new(FsmIDCondition::new_check(&check_cond)),
    }
}

fn parse_trigger_condition(trigger_cond: &configs::TriggerCondition) -> Box<dyn AsTriggerCondition> {
    match trigger_cond {
        TriggerCondition::LongswordLevelChanged { .. } => Box::new(LongswordCondition::new_trigger(&trigger_cond)),
        TriggerCondition::WeaponType { .. } => Box::new(WeaponTypeCondition::new_trigger(&trigger_cond)),
        TriggerCondition::QuestState { .. } => Box::new(QuestStateCondition::new_trigger(&trigger_cond)),
        TriggerCondition::Fsm { .. } => Box::new(FsmIDCondition::new_trigger(&trigger_cond)),
        TriggerCondition::InsectGlaiveLight { .. } => Box::new(InsectGlaiveCondition::new_trigger(&trigger_cond)),
        TriggerCondition::ChargeBlade { .. } => Box::new(ChargeBladeCondition::new_trigger(&trigger_cond)),
        TriggerCondition::UseItem { .. } => Box::new(UseItemCondition::new_trigger(&trigger_cond)),
    }
}

fn parse_event(event_cfg: &configs::Action) -> Option<Box<dyn AsAction>> {
    match event_cfg.cmd {
        configs::Command::SendChatMessage => Some(Box::new(SendChatMessageEvent::new(&event_cfg.param))),
        configs::Command::SystemMessage => None,
    }
}
