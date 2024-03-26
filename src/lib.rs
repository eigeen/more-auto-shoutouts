use std::sync::Once;

use log::{error, info};
use tokio::signal;
use tokio::sync::mpsc;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

use crate::event::Event;
use crate::triggers::TriggerManager;

mod conditions;
mod configs;
mod event;
mod game;
mod game_context;
mod handlers;
#[cfg(feature = "use_logger")]
mod logger;
mod triggers;

static MAIN_THREAD_ONCE: Once = Once::new();
// static mut TOKIO_RUNTIME: Option<Arc<Mutex<Runtime>>> = None;

struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }
}

#[cfg(feature = "use_logger")]
mod use_logger {
    use log::LevelFilter;
    use once_cell::sync::Lazy;

    use crate::logger::MASLogger;

    static LOGGER: Lazy<MASLogger> = Lazy::new(|| MASLogger::new());

    pub fn init_log() {
        log::set_logger(&*LOGGER).unwrap();
        log::set_max_level(LevelFilter::Debug);
    }
}

#[cfg(not(feature = "use_logger"))]
mod use_logger {
    pub fn init_log() {
        // no log backend
    }
}

use use_logger::init_log;

fn main_entry() -> Result<(), String> {
    init_log();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    info!("版本: {}", env!("CARGO_PKG_VERSION"));

    let _app = App::new();

    runtime.block_on(async {
        let (tx, rx) = mpsc::channel(128);
        // 事件处理器
        tokio::spawn(async move { handlers::event_handler(rx).await });
        // 事件监听器
        let tx1 = tx.clone();
        tokio::spawn(async move { handlers::event_listener(tx1).await });
        // 首次自动加载配置文件
        match handlers::load_triggers() {
            Ok(trigger_mgr) => {
                if let Err(e) = tx.send(Event::LoadTriggers { trigger_mgr }).await {
                    error!("加载配置失败：{}", e);
                };
            }
            Err(e) => {
                error!("加载配置失败：{}", e);
            }
        };

        // 于此处阻塞
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
                        error!("发生致命错误，已终止插件运行：{}", e);
                    };
                });
            });
        }
        DLL_PROCESS_DETACH => (),
        _ => (),
    }
    TRUE
}
