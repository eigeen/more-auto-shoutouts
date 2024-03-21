use log::error;

use crate::{
    configs::{CheckCondition, TriggerCondition},
    game_context::Context,
    triggers::{AsCheckCondition, AsTriggerCondition, Event},
};

pub struct WeaponTypeCondition {
    trigger_fn: Box<dyn Fn(&Event) -> bool + Send>,
    check_fn: Box<dyn Fn(&Context) -> bool + Send>,
}

impl WeaponTypeCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let cond = cond.clone();
        let trigger_fn: Box<dyn Fn(&Event) -> bool + Send> = if let TriggerCondition::WeaponType { value } = cond {
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
        }
    }

    pub fn new_check(cond: &CheckCondition) -> Self {
        let cond = cond.clone();
        let check_fn: Box<dyn Fn(&Context) -> bool + Send> = if let CheckCondition::WeaponType { value } = cond {
            Box::new(move |ctx| value == ctx.weapon_type.as_i32())
        } else {
            error!("internal: WeaponTypeCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        WeaponTypeCondition {
            trigger_fn: Box::new(|_| false),
            check_fn,
        }
    }
}

impl AsTriggerCondition for WeaponTypeCondition {
    fn check(&self, event: &Event) -> bool {
        (self.trigger_fn)(event)
    }
}

impl AsCheckCondition for WeaponTypeCondition {
    fn check(&self, ctx: &Context) -> bool {
        (self.check_fn)(ctx)
    }
}
