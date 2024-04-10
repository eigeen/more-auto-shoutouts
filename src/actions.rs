use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
};

use async_trait::async_trait;
use mhw_toolkit::game_util;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::configs;

static CHAT_MESSAGE_SENDER: Lazy<game_util::ChatMessageSender> = Lazy::new(|| game_util::ChatMessageSender::new());

pub type ActionContext = Arc<Mutex<HashMap<String, String>>>;

#[async_trait]
pub trait AsAction: Send + Sync {
    async fn execute(&self, context: &ActionContext);
    async fn reset(&self);
}

pub struct SendChatMessageAction {
    msg: String,
    cnt: AtomicI32,
    enabled_cnt: bool,
}

impl SendChatMessageAction {
    pub fn new(msg: &str, enabled_cnt: bool) -> Self {
        SendChatMessageAction {
            msg: msg.to_string(),
            cnt: AtomicI32::new(1),
            enabled_cnt,
        }
    }
}

#[async_trait]
impl AsAction for SendChatMessageAction {
    async fn execute(&self, action_ctx: &ActionContext) {
        let mut msg = self.msg.clone();
        for (key, value) in &*action_ctx.lock().await {
            let placeholder = format!("{{{{{}}}}}", key); // placeholder = "{{ key }}"
            msg = msg.replace(&placeholder, &value);
        }
        if self.enabled_cnt {
            msg = msg.replace("{{counter}}", &self.cnt.fetch_add(1, Ordering::SeqCst).to_string());
        }
        CHAT_MESSAGE_SENDER.send(&msg);
    }
    async fn reset(&self) {
        if self.enabled_cnt {
            self.cnt.store(1, Ordering::SeqCst);
        }
    }
}

pub fn create_action(action_cfg: &configs::Action, enable_cnt: bool) -> Option<Box<dyn AsAction>> {
    match action_cfg.cmd {
        configs::Command::SendChatMessage => Some(Box::new(SendChatMessageAction::new(&action_cfg.param, enable_cnt))),
    }
}
