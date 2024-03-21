use log::{debug, info};
use mhw_toolkit::{game_util, util};
use once_cell::sync::Lazy;

use crate::game_context::{ChargeBlade, ChatCommand, Fsm, InsectGlaive, SpecializedTool};

const QUEST_BASE: *const i32 = 0x14500CAF0 as *const i32;
const QUEST_OFFSETS: isize = 0x38;
const PLAYER_BASE: *const i32 = 0x145011760 as *const i32;
const PLAYER_FSMTARGET_OFFSETS: &[isize] = &[0x50, 0x6274];
const PLAYER_FSMID_OFFSETS: &[isize] = &[0x50, 0x6278];
const USE_ITEM_OFFSETS: &[isize] = &[0x50, 0x80, 0x80, 0x10, 0x288, 0x28E0];

const WEAPON_DATA_BASE: *const i32 = 0x145011760 as *const i32;
const LONGSWORD_OFFSETS: &[isize] = &[0x50, 0x468, 0x70, 0x10, 0x18, 0x2370];
const WEAPON_OFFSETS: &[isize] = &[0x50, 0xC0, 0x8, 0x78, 0x2E8];
const SPETIAL_TOOL_0_OFFSETS: &[isize] = &[0x50, 0xC0, 0x8, 0x78, 0x31C];
const SPETIAL_TOOL_1_OFFSETS: &[isize] = &[0x50, 0xC0, 0x8, 0x78, 0x320];
const WEAPON_DATA_OFFSETS: &[isize] = &[0x50, 0x76B0];

const CHARGE_BLADE_BASE: *const i32 = 0x1450EA510 as *const i32;
const CHARGE_BLADE_MAX_PHIALS_OFFSETS: &[isize] = &[0x110, 0x98, 0x58, 0x5F98];

const CHAT_COMMAND_PREFIX: &str = "!mas ";
static CHAT_MESSAGE_RECV: Lazy<game_util::ChatMessageReceiver> = Lazy::new(|| {
    let mut instance = game_util::ChatMessageReceiver::new();
    instance.set_prefix_filter(CHAT_COMMAND_PREFIX);
    instance
});

pub fn get_chat_command() -> Option<ChatCommand> {
    if let Some(msg) = CHAT_MESSAGE_RECV.try_recv() {
        debug!("接收用户命令消息：{}", msg);
        let cmd = ChatCommand::from_str(&msg[CHAT_COMMAND_PREFIX.len()..]);
        if cmd.is_none() {
            info!("无效的命令：{}", msg);
            game_util::send_chat_message("无效的命令");
        }
        cmd
    } else {
        None
    }
}

pub fn get_quest_state() -> i32 {
    match util::get_value_with_offset(QUEST_BASE, &[QUEST_OFFSETS]) {
        Some(qs) => qs,
        None => 0,
    }
}

pub fn get_longsword_level() -> i32 {
    util::get_value_with_offset(WEAPON_DATA_BASE, LONGSWORD_OFFSETS).unwrap_or(99)
}

pub fn get_weapon_type() -> i32 {
    match util::get_value_with_offset(WEAPON_DATA_BASE, WEAPON_OFFSETS) {
        Some(w) => w,
        None => 0,
    }
}

pub fn get_fsm() -> Fsm {
    let id = match util::get_value_with_offset(PLAYER_BASE, PLAYER_FSMID_OFFSETS) {
        Some(v) => v,
        None => 0,
    };
    let target = match util::get_value_with_offset(PLAYER_BASE, PLAYER_FSMTARGET_OFFSETS) {
        Some(v) => v,
        None => 0,
    };
    Fsm { id, target }
}

pub fn get_use_item_id() -> i32 {
    match util::get_value_with_offset(PLAYER_BASE, USE_ITEM_OFFSETS) {
        Some(v) => v,
        None => 0,
    }
}

pub fn get_insect_glaive_data() -> Option<InsectGlaive> {
    let weapon_data_base = match util::get_ptr_with_offset(WEAPON_DATA_BASE as *const f32, WEAPON_DATA_OFFSETS) {
        Some(ptr) => ptr,
        None => return None,
    };
    let data: InsectGlaive = InsectGlaive {
        attack_timer: util::get_value_with_offset(weapon_data_base, &[0x2368]).unwrap_or(0.0),
        speed_timer: util::get_value_with_offset(weapon_data_base, &[0x236C]).unwrap_or(0.0),
        defense_timer: util::get_value_with_offset(weapon_data_base, &[0x2370]).unwrap_or(0.0),
    };

    Some(data)
}

pub fn get_charge_blade_data() -> Option<ChargeBlade> {
    let weapon_data_base = match util::get_ptr_with_offset(WEAPON_DATA_BASE as *const f32, WEAPON_DATA_OFFSETS) {
        Some(ptr) => ptr,
        None => return None,
    };
    let data: ChargeBlade = ChargeBlade {
        sword_power: util::get_value_with_offset(weapon_data_base, &[0x2370]).unwrap_or(0.0),
        sword_charge_timer: util::get_value_with_offset(weapon_data_base, &[0x237C]).unwrap_or(0.0),
        shield_charge_timer: util::get_value_with_offset(weapon_data_base, &[0x2378]).unwrap_or(0.0),
        phials: util::get_value_with_offset(weapon_data_base as *const i32, &[0x2374]).unwrap_or(0),
        max_phials: util::get_value_with_offset(CHARGE_BLADE_BASE, CHARGE_BLADE_MAX_PHIALS_OFFSETS).unwrap_or(0),
        power_axe_mode: util::get_value_with_offset(weapon_data_base as *const i32, &[0x2474]).unwrap_or(0),
        power_axe_timer: util::get_value_with_offset(weapon_data_base, &[0x2470]).unwrap_or(0.0),
    };

    Some(data)
}

// TODO
pub fn get_specialized_tool() -> Option<SpecializedTool> {
    None
}
