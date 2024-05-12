use std::sync::Once;

use log::{error, info};
use tokio::signal;
use tokio::sync::mpsc;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

use crate::event::Event;
use crate::triggers::TriggerManager;

mod actions;
mod conditions;
mod configs;
mod event;
mod game;
mod game_context;
mod handlers;
mod triggers;

#[cfg(feature = "use_audio")]
mod audios;
#[cfg(feature = "hooks")]
mod hooks;
#[cfg(feature = "use_logger")]
mod logger;

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

    static LOGGER: Lazy<MASLogger> = Lazy::new(MASLogger::new);

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
    info!("版本: {}", env!("CARGO_PKG_VERSION"));

    let _app = App::new();

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|e| e.to_string())?;
    runtime.block_on(async {
        let (tx, rx) = mpsc::channel(1024);
        // 事件处理器
        tokio::spawn(async move { handlers::event_handler(rx).await });
        // 事件监听器
        let tx1 = tx.clone();
        tokio::spawn(async move { handlers::event_listener(tx1).await });
        // 钩子注册与钩子事件转发
        #[cfg(feature = "hooks")]
        {
            let hooks_rx = hooks::install_hooks();
            let tx2 = tx.clone();
            tokio::spawn(async move { hooks::event_forwarder(hooks_rx, tx2).await });
        }

        // 首次自动加载配置文件
        match handlers::load_triggers().await {
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
