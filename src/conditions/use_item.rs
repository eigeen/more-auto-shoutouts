use async_trait::async_trait;
use log::error;

use crate::{
    configs::TriggerCondition,
    event::{Event, EventType},
    triggers::AsTriggerCondition,
};

use super::TriggerFn;

pub struct UseItemCondition {
    trigger_fn: TriggerFn,
}

impl UseItemCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let cond = cond.clone();
        let trigger_fn: TriggerFn = if let TriggerCondition::UseItem { item_id } = cond {
            Box::new(move |event| {
                if let Event::UseItem {
                    item_id: using_item_id, ..
                } = event
                {
                    &item_id == using_item_id
                } else {
                    false
                }
            })
        } else {
            error!("internal: UseItemCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        UseItemCondition { trigger_fn }
    }
}

#[async_trait]
impl AsTriggerCondition for UseItemCondition {
    async fn check(&self, event: &Event) -> bool {
        (self.trigger_fn)(event)
    }

    fn event_type(&self) -> EventType {
        EventType::UseItem
    }
}
