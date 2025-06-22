// Re-export main functions for backward compatibility
pub use state::{load_initial_state, load_initial_state_with_player_count, load_initial_state_with_rng, load_initial_state_with_player_names};
pub use patches::{apply_patch_to_state, process_phases, apply_action, AnimationConfig, DiscreteUpdate, process_phases_with_animation, apply_action_with_animation};
pub use phase_validation::{validate_phase_definitions, validate_phase_state, PhaseValidationResult};

// Exports for simplified state (use alias to avoid name conflict)
pub use state_init::load_initial_state as load_initial_state_simplified;
pub use verbs::apply_verb;
pub use entity_ui::enhance_entities_with_ui;
pub use action_execution_v2::{execute_entity_interaction, execute_zone_interaction};

pub mod state;
pub mod state_init;
pub mod verbs;
pub mod grid;
pub mod hex;
pub mod hex_zone_authority;
pub mod path;
pub mod patches;
pub mod phase_validation;
pub mod zone_authority;
pub mod enhanced_phases;
pub mod zone_tier;
pub mod view_zones;
pub mod entity_ui;
pub mod action_execution_v2;
pub mod action_executor;
pub mod patch_optimizer;
pub mod entity_pool;