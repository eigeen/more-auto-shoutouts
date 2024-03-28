use log::error;

use crate::{
    configs::{CheckCondition, TriggerCondition},
    event::{Event, EventType},
    triggers::{AsCheckCondition, AsTriggerCondition, SharedContext},
};

use super::{CheckFn, TriggerFn};

pub struct FsmCondition {
    trigger_fn: TriggerFn,
    check_fn: CheckFn,
    shared_ctx: SharedContext,
}

impl FsmCondition {
    pub fn new_trigger(cond: &TriggerCondition, shared_ctx: SharedContext) -> Self {
        let cond = cond.clone();
        let trigger_fn: TriggerFn = if let TriggerCondition::Fsm { new, old } = cond {
            Box::new(move |event| {
                if let Event::FsmChanged {
                    new: e_new, old: e_old, ..
                } = event
                {
                    if let Some(new) = &new {
                        if new != e_new {
                            return false;
                        }
                    }
                    if let Some(old) = &old {
                        if old != e_old {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            })
        } else {
            error!("internal: FsmIDCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        FsmCondition {
            trigger_fn,
            check_fn: Box::new(|_| false),
            shared_ctx,
        }
    }

    pub fn new_check(cond: &CheckCondition, shared_ctx: SharedContext) -> Self {
        let cond = cond.clone();
        let check_fn: CheckFn = if let CheckCondition::Fsm { value } = cond {
            Box::new(move |ctx| value == ctx.fsm)
        } else {
            error!("internal: FsmIDCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        FsmCondition {
            trigger_fn: Box::new(|_| false),
            check_fn,
            shared_ctx,
        }
    }
}

impl AsTriggerCondition for FsmCondition {
    fn check(&self, event: &Event) -> bool {
        (self.trigger_fn)(event)
    }

    fn event_type(&self) -> EventType {
        EventType::FsmChanged
    }
}

impl AsCheckCondition for FsmCondition {
    fn check(&self) -> bool {
        (self.check_fn)(&self.shared_ctx.read().unwrap())
    }
}
