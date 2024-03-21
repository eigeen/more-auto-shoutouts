use log::error;

use crate::{
    configs::{NewOldValueCmp, TriggerCondition},
    triggers::{AsTriggerCondition, Event},
};

pub struct InsectGlaiveCondition {
    trigger_fn: Box<dyn Fn(&Event) -> bool + Send>,
}

impl InsectGlaiveCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let cond = cond.clone();
        let trigger_fn: Box<dyn Fn(&Event) -> bool + Send> =
            if let TriggerCondition::InsectGlaiveLight { red, white, yellow } = cond {
                Box::new(move |event| {
                    if let Event::InsectGlaive { ctx } = event {
                        compare_cfg_ctx(
                            &red,
                            ctx.insect_glaive.attack_timer,
                            ctx.last_ctx.as_ref().unwrap().insect_glaive.attack_timer,
                        ) && compare_cfg_ctx(
                            &white,
                            ctx.insect_glaive.speed_timer,
                            ctx.last_ctx.as_ref().unwrap().insect_glaive.speed_timer,
                        ) && compare_cfg_ctx(
                            &yellow,
                            ctx.insect_glaive.defense_timer,
                            ctx.last_ctx.as_ref().unwrap().insect_glaive.defense_timer,
                        )
                    } else {
                        false
                    }
                })
            } else {
                error!("internal: InsectGlaiveCondition cmp_fn 参数不正确");
                Box::new(|_| false)
            };

        InsectGlaiveCondition { trigger_fn }
    }
}

impl AsTriggerCondition for InsectGlaiveCondition {
    fn check(&self, event: &Event) -> bool {
        (self.trigger_fn)(event)
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
