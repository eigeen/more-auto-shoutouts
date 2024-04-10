use async_trait::async_trait;
use log::error;

use super::{CheckFn, TriggerFn};
use crate::{
    actions::ActionContext, configs::{CheckCondition, TriggerCondition}, event::{Event, EventType}, triggers::{AsCheckCondition, AsTriggerCondition, SharedContext}
};

pub struct WeaponTypeCondition {
    trigger_fn: TriggerFn,
    check_fn: CheckFn,
    shared_ctx: SharedContext,
}

impl WeaponTypeCondition {
    pub fn new_trigger(cond: &TriggerCondition, shared_ctx: SharedContext) -> Self {
        let cond = cond.clone();
        let trigger_fn: TriggerFn = if let TriggerCondition::WeaponType { value } = cond {
            Box::new(move |event| {
                if let Event::WeaponTypeChanged { new, .. } = event {
                    &value == &new.as_i32()
                } else {
                    false
                }
            })
        } else {
            error!("internal: WeaponTypeCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        WeaponTypeCondition {
            trigger_fn,
            check_fn: Box::new(|_| false),
            shared_ctx,
        }
    }

    pub fn new_check(cond: &CheckCondition, shared_ctx: SharedContext) -> Self {
        let cond = cond.clone();
        let check_fn: CheckFn = if let CheckCondition::WeaponType { value } = cond {
            Box::new(move |ctx| value == ctx.weapon_type.as_i32())
        } else {
            error!("internal: WeaponTypeCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        WeaponTypeCondition {
            trigger_fn: Box::new(|_| false),
            check_fn,
            shared_ctx,
        }
    }
}

#[async_trait]
impl AsTriggerCondition for WeaponTypeCondition {
    async fn check(&self, event: &Event, _action_ctx: &ActionContext) -> bool {
        (self.trigger_fn)(event)
    }

    fn event_type(&self) -> EventType {
        EventType::WeaponTypeChanged
    }
}

#[async_trait]
impl AsCheckCondition for WeaponTypeCondition {
    async fn check(&self, _action_ctx: &ActionContext) -> bool {
        (self.check_fn)(&*self.shared_ctx.read().await)
    }
}
