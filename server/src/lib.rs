// Library crate for bluefelt-core to enable testing
pub mod app_state;
pub mod bundle;
pub mod engine;
pub mod lobby;
pub mod utils;
pub mod shorthand;
pub mod validation;
pub mod message_format;
pub mod conditions;
pub mod reload_notifier;

pub use bundle::{Bundle, BundleMap};
pub use engine::{load_initial_state, apply_verb, apply_patch_to_state, process_phases, apply_action};
pub use lobby::LobbyMap;
pub use message_format::{MessageFormat, UpdateFormat};