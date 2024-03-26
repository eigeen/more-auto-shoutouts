use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc, Mutex,
    },
};

use chrono::{DateTime, Duration, Utc};
use log::debug;
use mhw_toolkit::game_util;
use once_cell::sync::Lazy;
use rand::Rng;

use crate::{
    conditions::{
        charge_blade::ChargeBladeCondition, damage::DamageCondition, fsm::FsmCondition,
        insect_glaive::InsectGlaiveCondition, longsword::LongswordCondition, quest_state::QuestStateCondition,
        use_item::UseItemCondition, weapon_id::WeaponTypeCondition,
    },
    configs::{self, ActionMode, TriggerCondition},
    event::{Event, EventType},
    game_context::Context,
};

static CHAT_MESSAGE_SENDER: Lazy<game_util::ChatMessageSender> = Lazy::new(|| game_util::ChatMessageSender::new());

pub trait AsTriggerCondition: Send + Sync {
    fn check(&self, event: &Event) -> bool;
    fn event_type(&self) -> EventType;
}

pub trait AsCheckCondition: Send + Sync {
    fn check(&self, context: &Context) -> bool;
}

pub trait AsAction: Send + Sync {
    fn execute(&self);
}

pub trait AsTrigger: Send + Sync {
    fn event_type(&self) -> EventType;
    fn on_event(&self, event: &Event);
}

type DoTriggerFn = Box<dyn Fn(&Event) + Send + Sync>;

pub struct TriggerBuilder {
    name: Option<String>,
    actions: Vec<Box<dyn AsAction>>,
    trigger_condition: Box<dyn AsTriggerCondition>,
    check_conditions: Vec<Box<dyn AsCheckCondition>>,
    action_mode: ActionMode,
    cooldown: Option<SingleCoolDown>,
    event_type: EventType,
    action_idx: AtomicI32,
}

impl TriggerBuilder {
    pub fn new(trigger_condition: Box<dyn AsTriggerCondition>) -> TriggerBuilder {
        let event_type = trigger_condition.event_type();
        TriggerBuilder {
            name: None,
            actions: Vec::new(),
            trigger_condition,
            check_conditions: Vec::new(),
            action_mode: ActionMode::SequentialAll,
            cooldown: None,
            event_type,
            action_idx: AtomicI32::new(0),
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    pub fn set_cooldown(&mut self, cooldown: SingleCoolDown) {
        self.cooldown = Some(cooldown);
    }

    pub fn set_action_mode(&mut self, action_mode: ActionMode) {
        self.action_mode = action_mode;
    }

    pub fn add_action(&mut self, event: Box<dyn AsAction>) {
        self.actions.push(event);
    }

    pub fn add_check_condition(&mut self, cond: Box<dyn AsCheckCondition>) {
        self.check_conditions.push(cond);
    }

    pub fn build(self) -> Trigger {
        let name = self.name.clone();
        let event_type = self.event_type.clone();
        let do_trigger_fn: DoTriggerFn = Box::new(move |event| {
            if !self.check_conditions(event) {
                return;
            }
            match self.action_mode {
                ActionMode::SequentialAll => self.actions.iter().for_each(|e| {
                    e.execute();
                }),
                ActionMode::SequentialOne => {
                    self.execute_next_action();
                }
                ActionMode::Random => {
                    self.execute_random_one();
                }
            }
        });

        Trigger {
            name,
            do_trigger_fn,
            event_type,
        }
    }

    fn check_conditions(&self, event: &Event) -> bool {
        // 状态重置条件判断
        if let ActionMode::SequentialOne = self.action_mode {
            if let Event::QuestStateChanged { new, old, .. } = event {
                // 进入据点或离开据点时
                if *new == 1 || *old == 1 {
                    // reset idx
                    self.action_idx.store(0, Ordering::SeqCst);
                    // reset cooldown
                    if let Some(cooldown) = &self.cooldown {
                        cooldown.reset();
                    }
                }
            }
        }
        // 判断延迟触发器
        // TODO
        // 判断触发器
        if !self.trigger_condition.check(event) {
            return false;
        }
        // 判断检查器
        let checked = self.check_conditions.iter().all(|c| c.check(&event.extract_ctx()));
        if !checked {
            return false;
        }
        // 判断冷却
        if let Some(cd) = &self.cooldown {
            if !cd.check_set() {
                return false;
            }
        };
        true
    }

    fn execute_next_action(&self) {
        let mut action_idx = self.action_idx.fetch_add(1, Ordering::SeqCst);
        if action_idx >= self.actions.len() as i32 {
            self.action_idx.store(1, Ordering::SeqCst);
            action_idx = 0;
        }
        self.actions[action_idx as usize].execute();
    }

    fn execute_random_one(&self) {
        let idx = rand::thread_rng().gen_range(0..self.actions.len());
        self.actions[idx].execute();
    }
}

pub struct Trigger {
    name: Option<String>,
    do_trigger_fn: DoTriggerFn,
    event_type: EventType,
}

impl std::fmt::Debug for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Trigger").field("name", &self.name).field("event_type", &self.event_type).finish()
    }
}

impl AsTrigger for Trigger {
    fn event_type(&self) -> EventType {
        self.event_type.clone()
    }

    fn on_event(&self, event: &Event) {
        (self.do_trigger_fn)(event)
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
    triggers: HashMap<EventType, Vec<Arc<Trigger>>>,
    all_triggers: Vec<Arc<Trigger>>,
}

impl std::fmt::Debug for TriggerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TriggerManager").field("triggers.len", &self.triggers.len()).finish()
    }
}

impl TriggerManager {
    pub fn new() -> Self {
        TriggerManager {
            triggers: HashMap::new(),
            all_triggers: Vec::new(),
        }
    }

    pub fn register_trigger(&mut self, trigger: Trigger) {
        let shared_trigger = Arc::new(trigger);
        self.triggers
            .entry(shared_trigger.event_type())
            .or_insert_with(Vec::new)
            .push(shared_trigger.clone());
        self.all_triggers.push(shared_trigger);
    }

    pub fn broadcast(&self, event: &Event) {
        self.all_triggers.iter().for_each(|trigger| trigger.on_event(event));
    }

    pub fn dispatch(&self, event: &Event) {
        if event.event_type() == EventType::Damage {
            log::debug!("dispatch Damage");
        }
        // 需要广播的消息
        if event.event_type() == EventType::QuestStateChanged || event.event_type() == EventType::Damage {
            log::debug!("broadcast");
            self.broadcast(event);
            return;
        }
        let events = self.triggers.get(&event.event_type());
        if let Some(events) = events {
            events.iter().for_each(|trigger| {
                trigger.on_event(event);
            })
        }
    }
}

/// 通过配置注册 Trigger
pub fn register_trigger(t_cfg: &configs::Trigger) -> Trigger {
    let t_cfg = t_cfg.clone();
    let action_mode = t_cfg.action_mode.unwrap_or(configs::ActionMode::SequentialAll);
    let t_cond = register_trigger_condition(&t_cfg.trigger_on);

    let mut builder = TriggerBuilder::new(t_cond);
    builder.set_action_mode(action_mode);
    if let Some(name) = &t_cfg.name {
        builder.set_name(name);
    }
    builder.set_cooldown(SingleCoolDown::new(t_cfg.cooldown.unwrap_or(0.0)));
    t_cfg.check.iter().map(register_check_condition).for_each(|c| builder.add_check_condition(c));
    t_cfg.action.iter().filter_map(register_action).for_each(|e| builder.add_action(e));

    let debug_name: &str = match &builder.name {
        Some(n) => n,
        None => "unnamed",
    };
    debug!(
        "注册 trigger `{}` check({}), action({})",
        debug_name,
        builder.check_conditions.len(),
        builder.actions.len()
    );

    builder.build()
}

fn register_check_condition(check_cond: &configs::CheckCondition) -> Box<dyn AsCheckCondition> {
    match check_cond {
        configs::CheckCondition::LongswordLevel { .. } => Box::new(LongswordCondition::new_check(&check_cond)),
        configs::CheckCondition::WeaponType { .. } => Box::new(WeaponTypeCondition::new_check(&check_cond)),
        configs::CheckCondition::QuestState { .. } => Box::new(QuestStateCondition::new_check(&check_cond)),
        configs::CheckCondition::Fsm { .. } => Box::new(FsmCondition::new_check(&check_cond)),
    }
}

fn register_trigger_condition(trigger_cond: &configs::TriggerCondition) -> Box<dyn AsTriggerCondition> {
    match trigger_cond {
        TriggerCondition::LongswordLevelChanged { .. } => Box::new(LongswordCondition::new_trigger(&trigger_cond)),
        TriggerCondition::WeaponType { .. } => Box::new(WeaponTypeCondition::new_trigger(&trigger_cond)),
        TriggerCondition::QuestState { .. } => Box::new(QuestStateCondition::new_trigger(&trigger_cond)),
        TriggerCondition::Fsm { .. } => Box::new(FsmCondition::new_trigger(&trigger_cond)),
        TriggerCondition::InsectGlaiveLight { .. } => Box::new(InsectGlaiveCondition::new_trigger(&trigger_cond)),
        TriggerCondition::ChargeBlade { .. } => Box::new(ChargeBladeCondition::new_trigger(&trigger_cond)),
        TriggerCondition::UseItem { .. } => Box::new(UseItemCondition::new_trigger(&trigger_cond)),
        TriggerCondition::Damage { .. } => Box::new(DamageCondition::new_trigger(&trigger_cond)),
    }
}

fn register_action(action_cfg: &configs::Action) -> Option<Box<dyn AsAction>> {
    match action_cfg.cmd {
        configs::Command::SendChatMessage => Some(Box::new(SendChatMessageEvent::new(&action_cfg.param))),
        configs::Command::SystemMessage => None,
    }
}

/// 单点冷却时间管理器
pub struct SingleCoolDown {
    /// 冷却时间（秒）
    cooldown: f32,
    /// 冷却记录，记录上次触发时间
    record: Mutex<Option<DateTime<Utc>>>,
}

impl SingleCoolDown {
    pub fn new(cooldown: f32) -> Self {
        Self {
            cooldown,
            record: Mutex::new(None),
        }
    }

    pub fn reset(&self) {
        let mut r = self.record.lock().unwrap();
        *r = None;
    }

    pub fn check_set(&self) -> bool {
        let now = Utc::now();
        let cd_dur = Duration::try_milliseconds((self.cooldown * 1000.0) as i64).unwrap_or_default();
        let mut record = self.record.lock().unwrap();
        let last_time = match *record {
            Some(record) => record,
            None => {
                let default_record = now - cd_dur;
                *record = Some(default_record);
                default_record
            }
        };

        let expected_expire_time = last_time + cd_dur;
        if expected_expire_time <= now {
            // cd已过期
            *record = Some(now);
            true
        } else {
            false
        }
    }
}
