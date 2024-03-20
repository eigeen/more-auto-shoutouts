use std::sync::Once;

use log::{debug, error, info, LevelFilter};
use mhw_toolkit::logger::MHWLogger;
use once_cell::sync::Lazy;
use snafu::prelude::*;
use tokio::signal;
use tokio::sync::mpsc;
use triggers::Trigger;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

use crate::triggers::TriggerManager;

mod conditions;
mod configs;
mod game;
mod game_context;
mod handlers;
mod triggers;

static MAIN_THREAD_ONCE: Once = Once::new();
// static mut TOKIO_RUNTIME: Option<Arc<Mutex<Runtime>>> = None;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Config error: {}", source))]
    Config { source: configs::ConfigError },
}

struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }
}

static LOGGER: Lazy<MHWLogger> = Lazy::new(|| MHWLogger::new("More Auto Shoutouts"));

fn init_log() {
    log::set_logger(&*LOGGER).unwrap();
    log::set_max_level(LevelFilter::Debug);
}

fn parse_config(cfg: &configs::Config) -> Vec<Trigger> {
    cfg.trigger.iter().map(|t| triggers::parse_trigger(t)).collect::<Vec<_>>()
}

fn main_entry() -> Result<(), Error> {
    init_log();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    info!("版本: {}", env!("CARGO_PKG_VERSION"));

    let _app = App::new();
    let config = configs::load_config("./nativePC/plugins/mas-config.toml").context(ConfigSnafu)?;
    debug!("load config: {:?}", config);
    info!("已加载配置文件");
    // 注册触发器
    let mut trigger_mgr = TriggerManager::new();
    parse_config(&config).into_iter().for_each(|trigger| {
        trigger_mgr.register_trigger(trigger);
    });

    runtime.block_on(async {
        let (tx, rx) = mpsc::channel(128);
        // 事件处理器
        tokio::spawn(async move { handlers::event_handler(rx, trigger_mgr).await });
        // 事件监听器
        tokio::spawn(async move { handlers::event_listener(tx).await });

        // block
        signal::ctrl_c().await.unwrap();
    });

    Ok(())
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "stdcall" fn DllMain(dll_module: HINSTANCE, call_reason: DWORD, reserved: LPVOID) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            MAIN_THREAD_ONCE.call_once(|| {
                std::thread::spawn(|| {
                    if let Err(e) = main_entry() {
                        error!("发生错误，已终止程序：{}", e);
                    };
                });
            });
        }
        DLL_PROCESS_DETACH => (),
        _ => (),
    }
    TRUE
}
