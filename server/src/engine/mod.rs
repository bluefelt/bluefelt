// Re-export main functions for backward compatibility
pub use state::{load_initial_state, load_initial_state_with_player_count, load_initial_state_with_rng, load_initial_state_with_player_names};
pub use verbs::apply_verb;
pub use patches::{apply_patch_to_state, process_phases, apply_action};

pub mod state;
pub mod verbs;
pub mod grid;
pub mod path;
pub mod patches;