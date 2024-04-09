use async_trait::async_trait;
use log::error;

use crate::{
    configs::{NewOldValueCmp, TriggerCondition, ValueCmp},
    event::{Event, EventType},
    triggers::{AsTriggerCondition, SharedContext},
};

pub struct ChargeBladeCondition {
    shared_ctx: SharedContext,
    sword_charge_timer: Option<NewOldValueCmp>,
    shield_charge_timer: Option<NewOldValueCmp>,
    power_axe_timer: Option<NewOldValueCmp>,
    phials: Option<NewOldValueCmp>,
    sword_power: Option<NewOldValueCmp>,
}

impl ChargeBladeCondition {
    pub fn new_trigger(cond: &TriggerCondition, shared_ctx: SharedContext) -> Self {
        let cond = cond.clone();
        if let TriggerCondition::ChargeBlade {
            sword_charge_timer,
            shield_charge_timer,
            power_axe_timer,
            phials,
            sword_power,
        } = cond
        {
            return ChargeBladeCondition {
                shared_ctx,
                sword_charge_timer,
                shield_charge_timer,
                power_axe_timer,
                phials,
                sword_power,
            };
        } else {
            error!("internal: InsectGlaiveCondition cmp_fn 参数不正确");
            panic!("internal: InsectGlaiveCondition cmp_fn 参数不正确");
        };
    }
}

#[async_trait]
impl AsTriggerCondition for ChargeBladeCondition {
    async fn check(&self, event: &Event) -> bool {
        let ctx = self.shared_ctx.read().await;
        if let Event::ChargeBlade = event {
            let phials = parse_cfg_phials_special(&self.phials, ctx.charge_blade.max_phials);
            let power_axe_timer = parse_cfg_power_axe_timer_special(&self.power_axe_timer);
            // 计算总电锯时长
            let new_total_power_axe_timer = ctx.charge_blade.phials as f32 * ctx.charge_blade.power_axe_timer;
            let old_total_power_axe_timer = ctx.last_ctx.as_ref().unwrap().charge_blade.phials as f32
                * ctx.last_ctx.as_ref().unwrap().charge_blade.power_axe_timer;
            compare_cfg_ctx_f32(
                &self.sword_charge_timer,
                ctx.charge_blade.sword_charge_timer,
                ctx.last_ctx.as_ref().unwrap().charge_blade.sword_charge_timer,
            ) && compare_cfg_ctx_f32(
                &self.shield_charge_timer,
                ctx.charge_blade.shield_charge_timer,
                ctx.last_ctx.as_ref().unwrap().charge_blade.shield_charge_timer,
            ) && compare_cfg_ctx(&phials, ctx.charge_blade.phials, ctx.last_ctx.as_ref().unwrap().charge_blade.phials)
                && compare_cfg_ctx_f32(
                    &self.sword_power,
                    ctx.charge_blade.sword_power,
                    ctx.last_ctx.as_ref().unwrap().charge_blade.sword_power,
                )
                && compare_cfg_ctx_f32(&power_axe_timer, new_total_power_axe_timer, old_total_power_axe_timer)
        } else {
            false
        }
    }

    fn event_type(&self) -> EventType {
        EventType::ChargeBlade
    }
}

fn compare_cfg_ctx_f32(cfg_value: &Option<NewOldValueCmp>, ctx_new: f32, ctx_old: f32) -> bool {
    if cfg_value.is_none() {
        return true;
    }
    let cfg_value = cfg_value.as_ref().unwrap();

    if cfg_value.new.is_none() && cfg_value.old.is_none() {
        return true;
    }
    if let Some(new) = &cfg_value.new {
        if *new != ctx_new as i32 {
            return false;
        }
    };
    if let Some(old) = &cfg_value.old {
        if *old != ctx_old as i32 {
            return false;
        }
    };
    true
}

fn compare_cfg_ctx(cfg_value: &Option<NewOldValueCmp>, ctx_new: i32, ctx_old: i32) -> bool {
    if cfg_value.is_none() {
        return true;
    }
    let cfg_value = cfg_value.as_ref().unwrap();

    if cfg_value.new.is_none() && cfg_value.old.is_none() {
        return true;
    }
    if let Some(new) = &cfg_value.new {
        if *new != ctx_new {
            return false;
        }
    };
    if let Some(old) = &cfg_value.old {
        if *old != ctx_old {
            return false;
        }
    };
    true
}

fn parse_cfg_phials_special(value: &Option<NewOldValueCmp>, max_phials: i32) -> Option<NewOldValueCmp> {
    if value.is_none() {
        return None;
    };
    let value = value.as_ref().unwrap();

    if let Some(new) = &value.new {
        if let ValueCmp::Special(s) = new {
            match s.as_str() {
                "full" => Some(NewOldValueCmp {
                    new: Some(ValueCmp::EqInt(max_phials)),
                    old: Some(ValueCmp::Cmp {
                        ne: Some(max_phials),
                        gt: None,
                        ge: None,
                        lt: None,
                        le: None,
                        r#in: None,
                        nin: None,
                    }),
                }),
                "empty" => Some(NewOldValueCmp {
                    new: Some(ValueCmp::EqInt(0)),
                    old: Some(ValueCmp::Cmp {
                        ne: Some(0),
                        gt: None,
                        ge: None,
                        lt: None,
                        le: None,
                        r#in: None,
                        nin: None,
                    }),
                }),
                other => {
                    error!("phials 不支持值 {}，已忽略该条件", other);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_cfg_power_axe_timer_special(value: &Option<NewOldValueCmp>) -> Option<NewOldValueCmp> {
    if value.is_none() {
        return None;
    };
    let value = value.as_ref().unwrap();

    if let Some(v_new) = &value.new {
        if let ValueCmp::Special(s) = v_new {
            match s.as_str() {
                "enabled" => Some(NewOldValueCmp {
                    new: Some(ValueCmp::Cmp {
                        ne: None,
                        gt: Some(0),
                        ge: None,
                        lt: None,
                        le: None,
                        r#in: None,
                        nin: None,
                    }),
                    old: Some(ValueCmp::Cmp {
                        ne: None,
                        gt: None,
                        ge: None,
                        lt: None,
                        le: Some(0),
                        r#in: None,
                        nin: None,
                    }),
                }),
                "disabled" => Some(NewOldValueCmp {
                    new: Some(ValueCmp::Cmp {
                        ne: None,
                        gt: None,
                        ge: None,
                        lt: None,
                        le: Some(0),
                        r#in: None,
                        nin: None,
                    }),
                    old: Some(ValueCmp::Cmp {
                        ne: None,
                        gt: Some(0),
                        ge: None,
                        lt: None,
                        le: None,
                        r#in: None,
                        nin: None,
                    }),
                }),
                other => {
                    error!("power_axe_timer 不支持值 {}，已忽略该条件", other);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    }
}
