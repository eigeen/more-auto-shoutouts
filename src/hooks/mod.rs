use std::sync::Mutex;

use log::error;
use tokio::sync::mpsc::{self, Receiver};

use crate::event::Event;

pub mod damage;

// static HOOKS_SENDER: Mutex<Option<mpsc::Sender<Event>>> = Mutex::new(None);
static HOOKS_SENDER: Mutex<Option<mpsc::Sender<Event>>> = Mutex::new(None);

pub fn init_hooks() -> Receiver<Event> {
    let (tx, rx) = mpsc::channel(256);
    HOOKS_SENDER.lock().unwrap().replace(tx);

    if let Err(_) = damage::init_hook() {
        error!("初始化伤害钩子错误");
    };

    rx
}

pub async fn event_forwarder(mut hooks_rx: Receiver<Event>, main_tx: tokio::sync::mpsc::Sender<Event>) {
    while let Some(event) = hooks_rx.recv().await {
        if let Err(e) = main_tx.send(event).await {
            error!("钩子消息转发失败：{}", e);
            return;
        }
    }
    error!("已终止钩子事件转发器")
}
