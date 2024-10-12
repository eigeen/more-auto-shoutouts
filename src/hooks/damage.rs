use std::ffi::{c_char, c_int};
use std::os::raw::c_void;
use std::ptr::{self, addr_of_mut};

use log::error;
use mhw_toolkit::game::address::{self, AddressRepository};
use winapi::shared::minwindef::BOOL;

use crate::event::Event;

use super::HOOKS_SENDER;

type DealDamageFunctionType = extern "C" fn(*mut c_void, c_int, *mut c_void, BOOL, BOOL, c_int, c_int, c_char, c_int);
static mut ORIGINAL_FUNCTION: *mut c_void = ptr::null_mut();

extern "C" fn hook_function(
    target: *mut c_void,
    damage: c_int,
    position: *mut c_void,
    is_tenderized: BOOL,
    is_crit: BOOL,
    unk0: c_int,
    unk1: c_int,
    unk2: c_char,
    attack_id: c_int,
) {
    // 获取伤害值
    if let Some(sender) = HOOKS_SENDER.lock().unwrap().as_ref() {
        if let Err(e) = sender.blocking_send(Event::Damage { damage }) {
            error!("发送伤害事件错误：{}", e);
        };
    }
    // 调用原始函数
    unsafe {
        let original: DealDamageFunctionType = std::mem::transmute(ORIGINAL_FUNCTION);
        original(target, damage, position, is_tenderized, is_crit, unk0, unk1, unk2, attack_id);
    }
}

pub fn install_hook() -> Result<(), String> {
    unsafe {
        // 初始化MinHook
        minhook_sys::MH_Initialize();

        // 获取目标函数地址
        let func_addr = AddressRepository::get_instance().lock().unwrap().get_address(address::player::DrawDamage)?;
        let target_function: *mut c_void = func_addr as *mut c_void;

        // 创建钩子
        let create_hook_status =
            minhook_sys::MH_CreateHook(target_function, hook_function as *mut c_void, addr_of_mut!(ORIGINAL_FUNCTION));

        if create_hook_status == minhook_sys::MH_OK {
            // 启用钩子
            minhook_sys::MH_EnableHook(target_function);
        } else {
            return Err(format!("创建伤害Hook失败：code: {}", create_hook_status));
        }

        minhook_sys::MH_ApplyQueued();
    }

    Ok(())
}
