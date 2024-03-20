use log::error;

use crate::{
    configs::{CheckCondition, TriggerCondition},
    game_context::Context,
    triggers::{AsCheckCondition, AsTriggerCondition, Event},
};

pub struct FsmIDCondition {
    trigger_fn: Box<dyn Fn(&Event) -> bool + Send>,
    check_fn: Box<dyn Fn(&Context) -> bool + Send>,
}

impl FsmIDCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let cond = cond.clone();
        let trigger_fn: Box<dyn Fn(&Event) -> bool + Send> = if let TriggerCondition::Fsm { new, old } = cond {
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

        FsmIDCondition {
            trigger_fn,
            check_fn: Box::new(|_| false),
        }
    }

    pub fn new_check(cond: &CheckCondition) -> Self {
        let cond = cond.clone();
        let check_fn: Box<dyn Fn(&Context) -> bool + Send> = if let CheckCondition::Fsm { value } = cond {
            Box::new(move |ctx| value == ctx.fsm)
        } else {
            error!("internal: FsmIDCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        FsmIDCondition {
            trigger_fn: Box::new(|_| false),
            check_fn,
        }
    }
}

impl AsTriggerCondition for FsmIDCondition {
    fn check(&self, event: &Event) -> bool {
        (self.trigger_fn)(event)
    }
}

impl AsCheckCondition for FsmIDCondition {
    fn check(&self, ctx: &Context) -> bool {
        (self.check_fn)(ctx)
    }
}
