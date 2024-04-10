use async_trait::async_trait;
use log::error;

use crate::{
    actions::ActionContext,
    configs::{NewOldValueCmp, TriggerCondition},
    event::{Event, EventType},
    triggers::{AsTriggerCondition, SharedContext},
};

pub struct InsectGlaiveCondition {
    shared_ctx: SharedContext,
    cond_red: Option<NewOldValueCmp>,
    cond_white: Option<NewOldValueCmp>,
    cond_yellow: Option<NewOldValueCmp>,
}

impl InsectGlaiveCondition {
    pub fn new_trigger(cond: &TriggerCondition, shared_ctx: SharedContext) -> Self {
        let cond = cond.clone();
        if let TriggerCondition::InsectGlaiveLight { red, white, yellow } = cond {
            InsectGlaiveCondition {
                shared_ctx,
                cond_red: red,
                cond_white: white,
                cond_yellow: yellow,
            }
        } else {
            error!("internal: InsectGlaiveCondition: invalid cond {:?}", cond);
            panic!("Invalid cond")
        }
    }
}

#[async_trait]
impl AsTriggerCondition for InsectGlaiveCondition {
    async fn check(&self, event: &Event, _action_ctx: &ActionContext) -> bool {
        let ctx = self.shared_ctx.read().await;
        if let Event::InsectGlaive = event {
            compare_cfg_ctx(
                &self.cond_red,
                ctx.insect_glaive.attack_timer,
                ctx.last_ctx.as_ref().unwrap().insect_glaive.attack_timer,
            ) && compare_cfg_ctx(
                &self.cond_white,
                ctx.insect_glaive.speed_timer,
                ctx.last_ctx.as_ref().unwrap().insect_glaive.speed_timer,
            ) && compare_cfg_ctx(
                &self.cond_yellow,
                ctx.insect_glaive.defense_timer,
                ctx.last_ctx.as_ref().unwrap().insect_glaive.defense_timer,
            )
        } else {
            false
        }
    }

    fn event_type(&self) -> EventType {
        EventType::InsectGlaive
    }
}

fn compare_cfg_ctx(cfg_value: &Option<NewOldValueCmp>, ctx_new: f32, ctx_old: f32) -> bool {
    if cfg_value.is_none() {
        return true;
    }
    let cfg_value = cfg_value.as_ref().unwrap();

    if cfg_value.new.is_none() && cfg_value.old.is_none() {
        return true;
    }
    if let Some(new) = &cfg_value.new {
        if *new != ctx_new as i32 {
            return false;
        }
    };
    if let Some(old) = &cfg_value.old {
        if *old != ctx_old as i32 {
            return false;
        }
    };
    true
}
