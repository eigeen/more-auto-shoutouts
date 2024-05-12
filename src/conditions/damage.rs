use std::time::Duration;

use async_trait::async_trait;
use log::error;

use crate::{
    actions::ActionContext,
    configs::{CheckCondition, FsmConfig, ValueCmp},
    game::DamageCollector,
    triggers::{AsCheckCondition, SharedContext},
};

pub struct DamageCondition {
    cond_damage: ValueCmp,
    cond_fsm: FsmConfig,
    cond_timeout: i32,
    cond_break_on_fsm_changed: bool,
    shared_ctx: SharedContext,
}

impl DamageCondition {
    pub fn new_check(cond: &CheckCondition, shared_ctx: SharedContext) -> Self {
        let cond = cond.clone();
        if let CheckCondition::Damage {
            damage,
            fsm,
            timeout,
            break_on_fsm_changed,
        } = cond
        {
            let timeout = timeout.unwrap_or(2000);
            DamageCondition {
                cond_damage: damage,
                cond_fsm: fsm,
                cond_timeout: timeout,
                cond_break_on_fsm_changed: break_on_fsm_changed,
                shared_ctx,
            }
        } else {
            error!("internal: DamageCondition cond 参数不正确");
            panic!("internal: DamageCondition cond 参数不正确");
        }
    }
}

#[async_trait]
impl AsCheckCondition for DamageCondition {
    async fn check(&self, action_ctx: &ActionContext) -> bool {
        let damage_collector = DamageCollector::instance();
        let now_fsm = self.shared_ctx.read().await.fsm;
        if self.cond_fsm == now_fsm {
            let damage = if self.cond_break_on_fsm_changed {
                damage_collector.collect_fsm(&now_fsm, Duration::from_millis(self.cond_timeout as u64)).await
            } else {
                damage_collector.collect_time(Duration::from_millis(self.cond_timeout as u64)).await
            };
            action_ctx.lock().await.insert("damage".to_string(), damage.to_string());
            self.cond_damage == damage
        } else {
            false
        }
    }
}
