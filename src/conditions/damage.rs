use std::collections::HashMap;

use async_trait::async_trait;
use log::error;
use tokio::sync::Mutex;

use crate::{
    configs::{TriggerCondition, ValueCmp},
    event::{Event, EventType},
    triggers::{ActionContext, AsTriggerCondition},
};

pub struct DamageCondition {
    trigger_value: ValueCmp,
    action_ctx: Mutex<ActionContext>,
}

impl DamageCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let mut instance = DamageCondition {
            trigger_value: ValueCmp::EqInt(0),
            action_ctx: Mutex::new(None),
        };

        let cond = cond.clone();
        if let TriggerCondition::Damage { value } = cond {
            instance.trigger_value = value;
        } else {
            error!("internal: DamageCondition cmp_fn 参数不正确");
        }

        instance
    }
}

#[async_trait]
impl AsTriggerCondition for DamageCondition {
    async fn check(&self, event: &Event) -> bool {
        if let Event::Damage { damage, .. } = event {
            if &self.trigger_value == damage {
                let mut context = HashMap::new();
                context.insert("damage".to_string(), damage.to_string());
                self.action_ctx.lock().await.replace(context);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn event_type(&self) -> EventType {
        EventType::Damage
    }

    async fn get_action_context(&self) -> ActionContext {
        self.action_ctx.lock().await.clone()
    }
}
