// Re-export main functions for backward compatibility
pub use state::load_initial_state;
pub use verbs::apply_verb;
pub use patches::{apply_patch_to_state, process_phases, apply_action};

pub mod state;
pub mod verbs;
pub mod grid;
pub mod path;
pub mod patches;