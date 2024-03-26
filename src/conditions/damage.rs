use log::error;

use super::TriggerFn;
use crate::{
    configs::TriggerCondition,
    event::{Event, EventType},
    triggers::AsTriggerCondition,
};

pub struct DamageCondition {
    trigger_fn: TriggerFn,
}

impl DamageCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let cond = cond.clone();
        let trigger_fn: TriggerFn = if let TriggerCondition::Damage { value } = cond {
            Box::new(move |event| {
                if let Event::Damage { damage } = event {
                    log::debug!("Event::Damage damage = {damage}");
                    &value == damage
                } else {
                    false
                }
            })
        } else {
            error!("internal: DamageCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        DamageCondition { trigger_fn }
    }
}

impl AsTriggerCondition for DamageCondition {
    fn check(&self, event: &Event) -> bool {
        (self.trigger_fn)(event)
    }

    fn event_type(&self) -> EventType {
        EventType::Damage
    }
}
