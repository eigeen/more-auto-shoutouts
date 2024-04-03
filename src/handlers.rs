use crate::{
    configs,
    event::Event,
    game_context::{ChargeBlade, ChatCommand, Context, InsectGlaive},
    triggers::{self, SharedContext, Trigger},
    tx_send_or_break, TriggerManager,
};

use log::{debug, error, info};
use mhw_toolkit::game_util::{self, WeaponType};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};

/// 事件监听器
pub async fn event_listener(tx: Sender<Event>) {
    let mut ctx = Context::default();
    loop {
        // 每秒20次事件检查
        tokio::time::sleep(Duration::from_millis(50)).await;
        // 更新上下文
        ctx.update_context();

        // 检查事件
        // 消息事件
        if let Some(cmd) = &ctx.chat_command {
            match cmd {
                ChatCommand::ReloadConfig => {
                    debug!("on {}", "ChatCommand::ReloadConfig");
                    info!("接收用户命令：{:?}", cmd);
                    let trigger_mgr = match load_triggers() {
                        Ok(mgr) => mgr,
                        Err(e) => {
                            error!("加载配置失败：{}", e);
                            continue;
                        }
                    };
                    game_util::show_game_message("已重载配置");
                    tx_send_or_break!(tx.send(Event::LoadTriggers { trigger_mgr }));
                }
                ChatCommand::Enable => {
                    debug!("on {}", "ChatCommand::Enable");
                    info!("接收用户命令：{:?}", cmd);
                    game_util::show_game_message("已启用插件");
                    ctx.plugin_enabled = true;
                }
                ChatCommand::Disable => {
                    debug!("on {}", "ChatCommand::Disable");
                    info!("接收用户命令：{:?}", cmd);
                    game_util::show_game_message("已禁用插件");
                    ctx.plugin_enabled = false;
                }
            }
        }
        if !ctx.plugin_enabled {
            continue;
        }

        // 只读部分
        let last_ctx = ctx.last_ctx.as_ref().unwrap();

        // 同步上下文
        tx_send_or_break!(tx.send(Event::UpdateContext { ctx: ctx.clone() }));

        if ctx.quest_state != last_ctx.quest_state {
            debug!("on {} from {:?} to {:?}", "Event::QuestStateChanged", last_ctx.quest_state, ctx.quest_state);
            tx_send_or_break!(tx.send(Event::QuestStateChanged {
                new: ctx.quest_state,
                old: last_ctx.quest_state,
            }));
        }
        if ctx.weapon_type != last_ctx.weapon_type {
            debug!("on {} from {:?} to {:?}", "Event::WeaponTypeChanged", last_ctx.weapon_type, ctx.weapon_type);
            tx_send_or_break!(tx.send(Event::WeaponTypeChanged {
                new: ctx.weapon_type,
                old: last_ctx.weapon_type,
            }));
        }
        if ctx.fsm != last_ctx.fsm {
            debug!("on {} from {:?} to {:?}", "Event::FsmChanged", last_ctx.fsm, ctx.fsm);
            tx_send_or_break!(tx.send(Event::FsmChanged {
                new: ctx.fsm,
                old: last_ctx.fsm,
            }));
        }
        if ctx.use_item_id > 0 && ctx.use_item_id < 3000 && ctx.use_item_id != last_ctx.use_item_id {
            debug!("on {} id = {}", "Event::UseItem", ctx.use_item_id);
            tx_send_or_break!(tx.send(Event::UseItem {
                item_id: ctx.use_item_id,
            }));
        }
        if WeaponType::LongSword == ctx.weapon_type {
            if ctx.longsword_level != last_ctx.longsword_level {
                debug!(
                    "on {} from {} to {}",
                    "Event::LongswordLevelChanged", last_ctx.longsword_level, ctx.longsword_level
                );
                tx_send_or_break!(tx.send(Event::LongswordLevelChanged {
                    new: ctx.longsword_level,
                    old: last_ctx.longsword_level,
                }));
            }
        } else if WeaponType::InsectGlaive == ctx.weapon_type {
            let new = &ctx.insect_glaive;
            let old = &last_ctx.insect_glaive;
            if is_insect_glaive_changed(new, old) {
                debug!("on {}", "Event::InsectGlaive",);
                tx_send_or_break!(tx.send(Event::InsectGlaive));
            }
        } else if WeaponType::ChargeBlade == ctx.weapon_type {
            let new = &ctx.charge_blade;
            let old = &last_ctx.charge_blade;
            if is_charge_blade_changed(new, old) {
                debug!("on {}", "Event::ChargeBlade",);
                tx_send_or_break!(tx.send(Event::ChargeBlade));
            }
        }
    }

    error!("主事件发送端已终止");
}

fn is_insect_glaive_changed(new: &InsectGlaive, old: &InsectGlaive) -> bool {
    (new.attack_timer <= 0.0 && old.attack_timer > 0.0)
        || (new.attack_timer > 0.0 && old.attack_timer <= 0.0)
        || (new.speed_timer <= 0.0 && old.speed_timer > 0.0)
        || (new.speed_timer > 0.0 && old.speed_timer <= 0.0)
        || (new.defense_timer <= 0.0 && old.defense_timer > 0.0)
        || (new.defense_timer > 0.0 && old.defense_timer <= 0.0)
}

fn is_charge_blade_changed(new: &ChargeBlade, old: &ChargeBlade) -> bool {
    (new.power_axe_timer <= 0.0 && old.power_axe_timer > 0.0)
        || (new.power_axe_timer > 0.0 && old.power_axe_timer <= 0.0)
        || (new.sword_charge_timer <= 0.0 && old.sword_charge_timer > 0.0)
        || (new.sword_charge_timer > 0.0 && old.sword_charge_timer <= 0.0)
        || (new.shield_charge_timer <= 0.0 && old.shield_charge_timer > 0.0)
        || (new.shield_charge_timer > 0.0 && old.shield_charge_timer <= 0.0)
        || (new.phials != old.phials)
        || (new.power_axe_mode != old.power_axe_mode)
        || (new.sword_power != old.sword_power)
}

/// 事件处理器
pub async fn event_handler(mut rx: Receiver<Event>) {
    let mut trigger_mgr: Option<TriggerManager> = None;
    loop {
        if let Some(e) = rx.recv().await {
            if let Event::LoadTriggers { trigger_mgr: mgr } = e {
                trigger_mgr = Some(mgr);
                info!("已加载新的TriggerManager");
                continue;
            }
            if let Some(mgr) = &mut trigger_mgr {
                if let Event::UpdateContext { ctx } = e {
                    mgr.update_ctx(&ctx);
                    continue;
                }
                // 处理
                mgr.dispatch(&e);
            }
        } else {
            error!("接收端错误");
            break;
        };
    }
}

pub fn load_triggers() -> Result<TriggerManager, String> {
    info!("尝试加载配置文件 nativePC/plugins/mas-config.toml");
    let config = match configs::load_config("./nativePC/plugins/mas-config.toml") {
        Ok(cfg) => cfg,
        Err(e) => return Err(e.to_string()),
    };
    debug!("load config: {:?}", config);
    info!("已加载配置文件");
    // 注册触发器
    let shared_ctx = Arc::new(std::sync::RwLock::new(Context::default()));
    let mut trigger_mgr = TriggerManager::new(shared_ctx.clone());
    parse_config(&config, shared_ctx.clone()).into_iter().for_each(|trigger| {
        trigger_mgr.register_trigger(trigger);
    });
    Ok(trigger_mgr)
}

pub fn parse_config(cfg: &configs::Config, shared_ctx: SharedContext) -> Vec<Trigger> {
    cfg.trigger
        .iter()
        .map(move |t| {
            let shared_ctx_clone = Arc::clone(&shared_ctx);
            triggers::register_trigger(t, shared_ctx_clone)
        })
        .collect::<Vec<_>>()
}

#[macro_export]
macro_rules! tx_send_or_break {
    ( $tx:expr ) => {
        if let Err(e) = $tx.await {
            error!("send event error: {}", e);
            break;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_insect_glaive_changed() {
        let mut old = InsectGlaive::default();
        let mut new = InsectGlaive::default();
        assert_eq!(is_insect_glaive_changed(&new, &old), false);

        new.attack_timer = 1.0;
        assert_eq!(is_insect_glaive_changed(&new, &old), true);

        old.attack_timer = new.attack_timer;
        new.attack_timer = 2.0;
        assert_eq!(is_insect_glaive_changed(&new, &old), false);
    }
}
