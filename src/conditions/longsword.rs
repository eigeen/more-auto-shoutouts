use log::error;

use crate::{
    configs::{CheckCondition, TriggerCondition},
    event::{Event, EventType},
    game_context::Context,
    triggers::{AsCheckCondition, AsTriggerCondition},
};

use super::{CheckFn, TriggerFn};

pub struct LongswordCondition {
    trigger_fn: TriggerFn,
    check_fn: CheckFn,
}

impl LongswordCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let cond = cond.clone();
        let trigger_fn: TriggerFn = if let TriggerCondition::LongswordLevelChanged { new, old } = cond {
            Box::new(move |event| {
                if let Event::LongswordLevelChanged {
                    new: new_event,
                    old: old_event,
                    ctx: _,
                } = event
                {
                    if let Some(new) = &new {
                        if new != new_event {
                            return false;
                        }
                    }
                    if let Some(old) = &old {
                        if old != old_event {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            })
        } else {
            error!("internal: LongswordCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        LongswordCondition {
            trigger_fn,
            check_fn: Box::new(|_| false),
        }
    }

    pub fn new_check(cond: &CheckCondition) -> Self {
        let cond = cond.clone();
        let check_fn: CheckFn = if let CheckCondition::LongswordLevel { value } = cond {
            Box::new(move |ctx| {
                if ctx.weapon_type != 3 {
                    return false;
                };
                value == ctx.longsword_level
            })
        } else {
            error!("internal: LongswordCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        LongswordCondition {
            trigger_fn: Box::new(|_| false),
            check_fn,
        }
    }
}

impl AsTriggerCondition for LongswordCondition {
    fn check(&self, event: &Event) -> bool {
        (self.trigger_fn)(event)
    }

    fn event_type(&self) -> EventType {
        EventType::LongswordLevelChanged
    }
}

impl AsCheckCondition for LongswordCondition {
    fn check(&self, ctx: &Context) -> bool {
        (self.check_fn)(ctx)
    }
}
