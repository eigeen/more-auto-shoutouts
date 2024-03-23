use std::sync::atomic::{AtomicI32, Ordering};

use chrono::{DateTime, Duration, Utc};
use log::debug;
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
    event::Event,
    game_context::Context,
};

static CHAT_MESSAGE_SENDER: Lazy<game_util::ChatMessageSender> = Lazy::new(|| game_util::ChatMessageSender::new());

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
    name: Option<String>,
    actions: Vec<Box<dyn AsAction>>,
    trigger_condition: Box<dyn AsTriggerCondition>,
    check_conditions: Vec<Box<dyn AsCheckCondition>>,
    action_mode: ActionMode,
    action_idx: AtomicI32,
    cooldown: Option<SingleCoolDown>,
}

impl Trigger {
    pub fn new(event_mode: ActionMode, trigger_condition: Box<dyn AsTriggerCondition>) -> Trigger {
        Trigger {
            name: None,
            actions: Vec::new(),
            trigger_condition,
            check_conditions: Vec::new(),
            action_mode: event_mode,
            action_idx: AtomicI32::new(0),
            cooldown: None,
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string())
    }

    pub fn set_cooldown(&mut self, cooldown: SingleCoolDown) {
        self.cooldown = Some(cooldown);
    }

    pub fn add_action(&mut self, event: Box<dyn AsAction>) {
        self.actions.push(event)
    }

    pub fn add_check_condition(&mut self, cond: Box<dyn AsCheckCondition>) {
        self.check_conditions.push(cond)
    }

    fn execute_next_event(&self) {
        let mut event_idx = self.action_idx.fetch_add(1, Ordering::SeqCst);
        if event_idx >= self.actions.len() as i32 {
            self.action_idx.store(1, Ordering::SeqCst);
            event_idx = 0;
        }
        self.actions[event_idx as usize].execute();
    }

    fn execute_random_one(&self) {
        let idx = rand::thread_rng().gen_range(0..self.actions.len());
        self.actions[idx].execute();
    }

    fn reset_action_idx(&self) {
        self.action_idx.store(0, Ordering::SeqCst);
    }

    pub fn process(&mut self, event: &Event) {
        // 全局条件判断
        if let Event::QuestStateChanged { new, old, .. } = event {
            // 进入据点或离开据点时
            if *new == 1 || *old == 1 {
                if let ActionMode::SequentialOne = self.action_mode {
                    self.reset_action_idx();
                }
            }
        }
        // 判断触发器
        if !self.trigger_condition.check(event) {
            return;
        }
        // 判断检查器
        let checked = self.check_conditions.iter().all(|c| c.check(&event.extract_ctx()));
        if !checked {
            return;
        }
        // 判断冷却
        if let Some(cd) = &mut self.cooldown {
            if !cd.check_set_cooldown() {
                return;
            }
        }
        // 执行行为
        match self.action_mode {
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

    pub fn process_all(&mut self, event: &Event) {
        self.triggers.iter_mut().for_each(|t| {
            t.process(event);
        });
    }
}

pub fn parse_trigger(t_cfg: &configs::Trigger) -> Trigger {
    let t_cfg = t_cfg.clone();
    let event_mode = t_cfg.action_mode.unwrap_or(configs::ActionMode::SequentialAll);
    let t_cond = parse_trigger_condition(&t_cfg.trigger_on);

    let mut t = Trigger::new(event_mode, t_cond);
    if let Some(name) = &t_cfg.name {
        t.set_name(name);
    }
    t.set_cooldown(SingleCoolDown::new(t_cfg.cooldown.unwrap_or(0.0)));
    t_cfg.check.iter().map(parse_check_condition).for_each(|c| t.add_check_condition(c));
    t_cfg.action.iter().filter_map(parse_event).for_each(|e| t.add_action(e));

    let debug_name: &str = match &t.name {
        Some(n) => n,
        None => "unnamed",
    };
    debug!("注册 trigger `{}` check({}), action({})", debug_name, t.check_conditions.len(), t.actions.len());

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

/// 单点冷却时间管理器
pub struct SingleCoolDown {
    /// 冷却时间（秒）
    cooldown: f32,
    /// 冷却记录，记录上次触发时间
    record: Option<DateTime<Utc>>,
}

impl SingleCoolDown {
    pub fn new(cooldown: f32) -> Self {
        Self { cooldown, record: None }
    }

    pub fn check_set_cooldown(&mut self) -> bool {
        let now = Utc::now();
        let cd_dur = Duration::try_milliseconds((self.cooldown * 1000.0) as i64).unwrap_or_default();
        let last_time = match self.record {
            Some(record) => record,
            None => {
                let default_record = now - cd_dur;
                self.record = Some(default_record);
                default_record
            }
        };

        let expected_expire_time = last_time + cd_dur;
        if expected_expire_time <= now {
            // cd已过期
            self.record = Some(now);
            true
        } else {
            false
        }
    }
}
