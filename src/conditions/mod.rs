pub mod charge_blade;
pub mod damage;
pub mod fsm;
pub mod insect_glaive;
pub mod longsword;
pub mod quest_state;
pub mod use_item;
pub mod weapon_id;

type TriggerFn = Box<dyn Fn(&crate::event::Event) -> bool + Send + Sync>;
type CheckFn = Box<dyn Fn(&crate::game_context::Context) -> bool + Send + Sync>;
