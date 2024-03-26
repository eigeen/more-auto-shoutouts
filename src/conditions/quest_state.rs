use log::error;

use crate::{
    configs::{CheckCondition, TriggerCondition, ValueCmp},
    event::{Event, EventType},
    game_context::Context,
    triggers::{AsCheckCondition, AsTriggerCondition},
};

use super::{CheckFn, TriggerFn};

pub struct QuestStateCondition {
    trigger_fn: TriggerFn,
    check_fn: CheckFn,
}

impl QuestStateCondition {
    pub fn new_trigger(cond: &TriggerCondition) -> Self {
        let cond = cond.clone();
        let trigger_fn: TriggerFn = if let TriggerCondition::QuestState { value } = cond {
            let value = if let ValueCmp::Special(s) = value {
                match s.as_str() {
                    "join" => ValueCmp::EqInt(2),
                    "leaved" => ValueCmp::EqInt(1),
                    "success" => ValueCmp::EqInt(3),
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
        let check_fn: CheckFn = if let CheckCondition::QuestState { value } = cond {
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

    fn event_type(&self) -> EventType {
        EventType::QuestStateChanged
    }
}

impl AsCheckCondition for QuestStateCondition {
    fn check(&self, ctx: &Context) -> bool {
        (self.check_fn)(ctx)
    }
}
