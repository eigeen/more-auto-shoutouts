use log::error;

use crate::{
    configs::{CheckCondition, TriggerCondition, ValueCmp},
    event::Event,
    game_context::Context,
    triggers::{AsCheckCondition, AsTriggerCondition},
};

pub struct QuestStateCondition {
    trigger_fn: Box<dyn Fn(&Event) -> bool + Send>,
    check_fn: Box<dyn Fn(&Context) -> bool + Send>,
}

impl QuestStateCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let cond = cond.clone();
        let trigger_fn: Box<dyn Fn(&Event) -> bool + Send> = if let TriggerCondition::QuestState { value } = cond {
            let value = if let ValueCmp::Special(s) = value {
                match s.as_str() {
                    "join" => ValueCmp::EqInt(2),
                    "leaved" => ValueCmp::EqInt(1),
                    other => {
                        error!("QuestStateCondition 值{}无定义，已拒绝条件", other);
                        return QuestStateCondition {
                            trigger_fn: Box::new(|_| false),
                            check_fn: Box::new(|_| false),
                        };
                    }
                }
            } else {
                value
            };
            Box::new(move |event| {
                if let Event::QuestStateChanged { new, .. } = event {
                    &value == new
                } else {
                    false
                }
            })
        } else {
            error!("internal: QuestStateCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        QuestStateCondition {
            trigger_fn,
            check_fn: Box::new(|_| false),
        }
    }

    pub fn new_check(cond: &CheckCondition) -> Self {
        let cond = cond.clone();
        let check_fn: Box<dyn Fn(&Context) -> bool + Send> = if let CheckCondition::QuestState { value } = cond {
            Box::new(move |ctx| value == ctx.quest_state)
        } else {
            error!("internal: QuestStateCondition cmp_fn 参数不正确");
            Box::new(|_| false)
        };

        QuestStateCondition {
            trigger_fn: Box::new(|_| false),
            check_fn,
        }
    }
}

impl AsTriggerCondition for QuestStateCondition {
    fn check(&self, event: &Event) -> bool {
        (self.trigger_fn)(event)
    }
}

impl AsCheckCondition for QuestStateCondition {
    fn check(&self, ctx: &Context) -> bool {
        (self.check_fn)(ctx)
    }
}
