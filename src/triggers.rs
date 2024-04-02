use std::{
    collections::HashMap,
    sync::{
        atomic::{self, AtomicI32, Ordering},
        Arc, Mutex, RwLock,
    },
};

use chrono::{DateTime, Duration, Utc};
use log::{debug, error};
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

pub type ActionContext = Option<HashMap<String, String>>;
pub type SharedContext = Arc<RwLock<Context>>;

static CHAT_MESSAGE_SENDER: Lazy<game_util::ChatMessageSender> = Lazy::new(|| game_util::ChatMessageSender::new());

pub trait AsTriggerCondition: Send + Sync {
    fn check(&self, event: &Event) -> bool;
    fn event_type(&self) -> EventType;
    fn get_action_context(&self) -> ActionContext {
        None
    }
}

pub trait AsCheckCondition: Send + Sync {
    fn check(&self) -> bool;
}

pub trait AsAction: Send + Sync {
    fn execute(&self, context: &ActionContext);
    fn reset(&self);
}

pub trait AsTrigger: Send + Sync {
    fn event_type(&self) -> EventType;
    fn on_event(&mut self, event: &Event);
    fn on_event_reset(&mut self);
}

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

/// wrapper of `TriggerBuilder`
pub struct TriggerFns {
    builder: TriggerBuilder,
}

impl TriggerFns {

    pub fn new(builder: TriggerBuilder) -> Self {
        Self { builder }
    }

    pub fn execute(&mut self, event: &Event) {
        if !self.builder.check_conditions(event) {
            return;
        }
        let action_ctx = self.builder.trigger_condition.get_action_context();
        match self.builder.action_mode {
            ActionMode::SequentialAll => self.builder.actions.iter().for_each(|e| {
                e.execute(&action_ctx);
            }),
            ActionMode::SequentialOne => {
                self.builder.execute_next_action(&action_ctx);
            }
            ActionMode::Random => {
                self.builder.execute_random_one(&action_ctx);
            }
        }
    }

    pub fn reset(&mut self) {
        match self.builder.action_mode {
            ActionMode::SequentialAll => self.builder.actions.iter().for_each(|e| {
                e.reset();
            }),
            _ => {}
        }
    }
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
        let trigger_fns = TriggerFns::new(self);

        Trigger {
            name,
            trigger_fns,
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
        let checked = self.check_conditions.iter().all(|c| c.check());
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

    fn execute_next_action(&self, action_ctx: &ActionContext) {
        let mut action_idx = self.action_idx.fetch_add(1, Ordering::SeqCst);
        if action_idx >= self.actions.len() as i32 {
            self.action_idx.store(1, Ordering::SeqCst);
            action_idx = 0;
        }
        self.actions[action_idx as usize].execute(action_ctx);
    }

    fn execute_random_one(&self, action_ctx: &ActionContext) {
        let idx = rand::thread_rng().gen_range(0..self.actions.len());
        self.actions[idx].execute(action_ctx);
    }
}

pub struct Trigger {
    name: Option<String>,
    trigger_fns: TriggerFns,
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

    fn on_event(&mut self, event: &Event) {
        self.trigger_fns.execute(event)
    }

    /// 重置触发器触发次数
    fn on_event_reset(&mut self) {
        self.trigger_fns.reset()
    }
}

pub struct SendChatMessageEvent {
    msg: String,
    cnt: AtomicI32,
}

impl SendChatMessageEvent {
    pub fn new(msg: &str, enabled_cnt: bool) -> Self {
        SendChatMessageEvent { 
            msg: msg.to_string(),
            cnt: AtomicI32::new({
                if enabled_cnt { -1 } else { 1 }
            })
        }
    }
}

impl AsAction for SendChatMessageEvent {
    fn execute(&self, action_ctx: &ActionContext) {
        let mut msg = self.msg.clone();
        if let Some(context) = action_ctx {
            for (key, value) in context {
                let placeholder = format!("{{{{{}}}}}", key); // placeholder = "{{ key }}"
                msg = msg.replace(&placeholder, value);
            }
        };
        let cnt = self.cnt.load(atomic::Ordering::Relaxed);
        if cnt >= 1 {
            msg = msg.replace("%d", &self.cnt.fetch_add(1, atomic::Ordering::SeqCst).to_string());
        }
        CHAT_MESSAGE_SENDER.send(&msg);
    }
    fn reset(&self) {
        self.cnt.store(1, atomic::Ordering::SeqCst);
    }
}

/// 触发器管理
pub struct TriggerManager {
    triggers: HashMap<EventType, Vec<Arc<Mutex<Trigger>>>>,
    all_triggers: Vec<Arc<Mutex<Trigger>>>,
    shared_ctx: Arc<RwLock<Context>>,
}

impl std::fmt::Debug for TriggerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TriggerManager").field("triggers.len", &self.triggers.len()).finish()
    }
}

impl TriggerManager {
    pub fn new(shared_ctx: SharedContext) -> Self {
        TriggerManager {
            triggers: HashMap::new(),
            all_triggers: Vec::new(),
            shared_ctx,
        }
    }

    pub fn register_trigger(&mut self, trigger: Trigger) {
        let shared_trigger = Arc::new(Mutex::new(trigger));
        match shared_trigger.lock() {
            Ok(locked) => {
                self.triggers
                .entry(locked.event_type())
                .or_insert_with(Vec::new)
                .push(shared_trigger.clone());
            }
            Err(msg) => {
                error!("lock error happened in register_trigger fn, {}", msg);
            }
        };
        self.all_triggers.push(shared_trigger);
    }

    pub fn broadcast(&self, event: &Event) {
        self.all_triggers.iter().for_each(|trigger| trigger.lock().expect("").on_event(event));
    }

    pub fn broadcast_and_reset(&self, event: &Event) {
        self.all_triggers.iter().for_each(|trigger| {
            match trigger.lock() {
                Ok(mut locked) => {
                    locked.on_event(event);
                    locked.on_event_reset();
                }
                Err(msg) => {
                    error!("lock error happened in broadcast_and_reset fn, {}", msg);
                }
            };
        });
    }

    pub fn update_ctx(&self, ctx: &Context) {
        let mut shared_ctx = self.shared_ctx.write().unwrap();
        *shared_ctx = ctx.clone()
    }

    pub fn dispatch(&mut self, event: &Event) {
        // 需要广播的消息
        match event.event_type() {
            EventType::QuestStateChanged => {
                self.broadcast_and_reset(event);
                return;
            },
            EventType::Damage => {
                self.broadcast(event);
                return;
            }
            _ => {}
        }
        let triggers = self.triggers.get_mut(&event.event_type());
        if let Some(triggers) = triggers {
            triggers.iter_mut().for_each(|trigger| {
                match trigger.lock() {
                    Ok(mut locked) => {
                        locked.on_event(event);
                    }
                    Err(msg) => {
                        error!("lock error happened in dispatch fn, {}", msg);
                    }
                }
            })
        }
    }
}

/// 通过配置注册 Trigger
pub fn register_trigger(t_cfg: &configs::Trigger, shared_ctx: SharedContext) -> Trigger {
    let t_cfg = t_cfg.clone();
    let action_mode = t_cfg.action_mode.unwrap_or(configs::ActionMode::SequentialAll);
    let t_cond = register_trigger_condition(&t_cfg.trigger_on, shared_ctx.clone());

    let mut builder = TriggerBuilder::new(t_cond);
    builder.set_action_mode(action_mode);
    if let Some(name) = &t_cfg.name {
        builder.set_name(name);
    }
    builder.set_cooldown(SingleCoolDown::new(t_cfg.cooldown.unwrap_or(0.0)));
    t_cfg
        .check
        .iter()
        .map(|check_cond| register_check_condition(check_cond, shared_ctx.clone()))
        .for_each(|c| builder.add_check_condition(c));

    t_cfg.action.iter().filter_map(|item| match t_cfg.enable_cnt {
        Some(true) => register_action(item, true),
        _ => register_action(item, false)
    }).for_each(|e| builder.add_action(e));

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

fn register_check_condition(
    check_cond: &configs::CheckCondition,
    shared_ctx: SharedContext,
) -> Box<dyn AsCheckCondition> {
    match check_cond {
        configs::CheckCondition::LongswordLevel { .. } => {
            Box::new(LongswordCondition::new_check(&check_cond, shared_ctx))
        }
        configs::CheckCondition::WeaponType { .. } => Box::new(WeaponTypeCondition::new_check(&check_cond, shared_ctx)),
        configs::CheckCondition::QuestState { .. } => Box::new(QuestStateCondition::new_check(&check_cond, shared_ctx)),
        configs::CheckCondition::Fsm { .. } => Box::new(FsmCondition::new_check(&check_cond, shared_ctx)),
    }
}

fn register_trigger_condition(
    trigger_cond: &configs::TriggerCondition,
    shared_ctx: SharedContext,
) -> Box<dyn AsTriggerCondition> {
    match trigger_cond {
        TriggerCondition::LongswordLevelChanged { .. } => {
            Box::new(LongswordCondition::new_trigger(&trigger_cond, shared_ctx))
        }
        TriggerCondition::WeaponType { .. } => Box::new(WeaponTypeCondition::new_trigger(&trigger_cond, shared_ctx)),
        TriggerCondition::QuestState { .. } => Box::new(QuestStateCondition::new_trigger(&trigger_cond, shared_ctx)),
        TriggerCondition::Fsm { .. } => Box::new(FsmCondition::new_trigger(&trigger_cond, shared_ctx)),
        TriggerCondition::InsectGlaiveLight { .. } => {
            Box::new(InsectGlaiveCondition::new_trigger(&trigger_cond, shared_ctx))
        }
        TriggerCondition::ChargeBlade { .. } => Box::new(ChargeBladeCondition::new_trigger(&trigger_cond, shared_ctx)),
        TriggerCondition::UseItem { .. } => Box::new(UseItemCondition::new_trigger(&trigger_cond)),
        TriggerCondition::Damage { .. } => Box::new(DamageCondition::new_trigger(&trigger_cond)),
    }
}

fn register_action(action_cfg: &configs::Action, enabled_cnt: bool) -> Option<Box<dyn AsAction>> {
    match action_cfg.cmd {
        configs::Command::SendChatMessage => Some(Box::new(SendChatMessageEvent::new(&action_cfg.param, enabled_cnt))),
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
