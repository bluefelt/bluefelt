//! Core types and implementation for the action executor

use crate::bundle::Bundle;
use crate::conditions;
use crate::engine::patches::{replace_template_vars, replace_actor_template};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Core action execution engine
pub struct ActionExecutor {
    /// The game bundle containing action definitions
    pub bundle: Arc<Bundle>,
    /// Maximum recursion depth to prevent infinite loops
    pub max_depth: usize,
    /// Registry of verb executors
    verb_registry: HashMap<String, Box<dyn VerbExecutor>>,
}

/// Action execution context that flows through the pipeline
#[derive(Clone, Debug)]
pub struct ActionContext {
    /// Current player/actor executing the action
    pub actor: String,
    /// Current recursion depth
    pub depth: usize,
    /// Action arguments
    pub args: Value,
    /// Additional template variables
    pub template_vars: HashMap<String, String>,
    /// Parent context for debugging/tracing
    pub parent_context: Option<Box<ActionContext>>,
    /// Unique ID for this execution chain
    pub execution_id: String,
}

/// Result of action execution
#[derive(Debug)]
pub struct ActionResult {
    /// State patches to apply
    pub patches: Vec<Value>,
    /// Actions triggered by this execution
    pub triggered_actions: Vec<TriggeredAction>,
    /// Execution logs for debugging
    pub logs: Vec<ActionLog>,
    /// Performance metrics
    pub metrics: ActionMetrics,
}

/// Represents an action that should be executed
#[derive(Debug)]
pub struct TriggeredAction {
    /// ID of the action to execute
    pub action_id: String,
    /// Execution context
    pub context: ActionContext,
    /// Where this action was triggered from
    pub source: ActionSource,
    /// Priority for execution ordering (higher = first)
    pub priority: i32,
}

/// Source of a triggered action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionSource {
    /// Direct user action
    UserAction,
    /// "then" field of another action
    ThenAction,
    /// conditionalAction verb
    ConditionalAction,
    /// Phase transition
    PhaseTransition,
    /// Action trigger
    Trigger,
    /// Multi-step action
    MultiStep,
}

/// Log entry for action execution
#[derive(Debug)]
pub struct ActionLog {
    /// When the action was executed
    pub timestamp: u64,
    /// Action ID
    pub action_id: String,
    /// Verb used
    pub verb: String,
    /// Execution context
    pub context: ActionContext,
    /// Result of execution
    pub result: ActionLogResult,
}

/// Result of an action execution for logging
#[derive(Debug)]
pub enum ActionLogResult {
    /// Action executed successfully
    Success { patches: usize },
    /// Action skipped due to conditions
    Skipped { reason: String },
    /// Action failed with error
    Failed { error: String },
}

/// Performance metrics for action execution
#[derive(Debug, Default)]
pub struct ActionMetrics {
    /// Total number of actions executed
    pub total_actions: usize,
    /// Total number of patches generated
    pub total_patches: usize,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Maximum recursion depth reached
    pub max_depth_reached: usize,
}

/// Trait for verb executors
pub trait VerbExecutor: Send + Sync {
    /// Execute the verb with the given arguments
    fn execute(
        &self,
        state: &mut Value,
        args: &Value,
        context: &ActionContext,
        executor: &ActionExecutor,
    ) -> Result<ActionResult, String>;
}

impl ActionContext {
    /// Create a new root context
    pub fn new(actor: String, args: Value) -> Self {
        Self {
            actor,
            depth: 0,
            args,
            template_vars: HashMap::new(),
            parent_context: None,
            execution_id: generate_execution_id(),
        }
    }
    
    /// Create a child context with increased depth
    pub fn child(&self) -> Self {
        Self {
            actor: self.actor.clone(),
            depth: self.depth + 1,
            args: json!({}),
            template_vars: self.template_vars.clone(),
            parent_context: Some(Box::new(self.clone())),
            execution_id: self.execution_id.clone(),
        }
    }
    
    /// Create a child context with new arguments
    pub fn child_with_args(&self, args: Value) -> Self {
        Self {
            actor: self.actor.clone(),
            depth: self.depth + 1,
            args,
            template_vars: self.template_vars.clone(),
            parent_context: Some(Box::new(self.clone())),
            execution_id: self.execution_id.clone(),
        }
    }
    
    /// Add a template variable
    pub fn with_var(mut self, key: String, value: String) -> Self {
        self.template_vars.insert(key, value);
        self
    }
}

impl ActionResult {
    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            patches: vec![],
            triggered_actions: vec![],
            logs: vec![],
            metrics: ActionMetrics::default(),
        }
    }
    
    /// Merge another result into this one
    pub fn merge(&mut self, other: ActionResult) {
        self.patches.extend(other.patches);
        self.triggered_actions.extend(other.triggered_actions);
        self.logs.extend(other.logs);
        self.metrics.total_actions += other.metrics.total_actions;
        self.metrics.total_patches += other.metrics.total_patches;
        self.metrics.execution_time_ms += other.metrics.execution_time_ms;
        self.metrics.max_depth_reached = self.metrics.max_depth_reached.max(other.metrics.max_depth_reached);
    }
}

impl ActionExecutor {
    /// Create a new action executor
    pub fn new(bundle: Arc<Bundle>) -> Self {
        Self {
            bundle,
            max_depth: 20,
            verb_registry: HashMap::new(),
        }
    }
    
    /// Create with custom max depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }
    
    /// Register a verb executor
    pub fn register_verb(&mut self, verb: &str, executor: Box<dyn VerbExecutor>) {
        self.verb_registry.insert(verb.to_string(), executor);
    }
    
    /// Execute an action by ID
    pub fn execute_action(
        &self,
        state: &mut Value,
        action_id: &str,
        context: ActionContext,
    ) -> Result<ActionResult, String> {
        let start_time = Instant::now();
        
        // Check recursion depth
        if context.depth > self.max_depth {
            return Err(format!(
                "Max action depth {} exceeded at action '{}'", 
                self.max_depth, action_id
            ));
        }
        
        // Resolve action definition
        let action_def = self.resolve_action(action_id)?;
        
        // Process templates in the entire action definition
        let processed_action = self.process_templates(&action_def, state, &context);
        
        // Check "when" conditions
        if !self.check_conditions(&processed_action, state, &context)? {
            let log = ActionLog {
                timestamp: current_timestamp(),
                action_id: action_id.to_string(),
                verb: String::new(),
                context: context.clone(),
                result: ActionLogResult::Skipped { 
                    reason: "Conditions not met".to_string() 
                },
            };
            
            let mut result = ActionResult::empty();
            result.logs.push(log);
            result.metrics.execution_time_ms = start_time.elapsed().as_millis() as u64;
            return Ok(result);
        }
        
        // Extract verb
        let verb = processed_action.get("uses")
            .or_else(|| processed_action.get("verb"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Action '{}' missing verb/uses", action_id))?;
        
        // Extract args
        let args = processed_action.get("with")
            .cloned()
            .unwrap_or(json!({}));
        
        // Merge with context args
        let final_args = self.merge_args(&args, &context.args);
        
        // Execute the verb
        let mut result = self.execute_verb(verb, state, &final_args, &context)?;
        
        // Process "then" actions
        if let Some(then_actions) = processed_action.get("then").and_then(|t| t.as_array()) {
            for (i, then_action) in then_actions.iter().enumerate() {
                if let Some(then_id) = then_action.get("action").and_then(|a| a.as_str()) {
                    let then_args = then_action.get("with").cloned().unwrap_or(json!({}));
                    result.triggered_actions.push(TriggeredAction {
                        action_id: then_id.to_string(),
                        context: context.child_with_args(then_args),
                        source: ActionSource::ThenAction,
                        priority: 100 - i as i32, // Higher priority for earlier actions
                    });
                }
            }
        }
        
        // Process triggers
        if let Some(triggers) = processed_action.get("triggers").and_then(|t| t.as_array()) {
            for (i, trigger) in triggers.iter().enumerate() {
                self.process_trigger(trigger, state, &context, &mut result, i)?;
            }
        }
        
        // Log execution
        let log = ActionLog {
            timestamp: current_timestamp(),
            action_id: action_id.to_string(),
            verb: verb.to_string(),
            context: context.clone(),
            result: ActionLogResult::Success { 
                patches: result.patches.len() 
            },
        };
        result.logs.push(log);
        
        // Update metrics
        result.metrics.total_actions = 1;
        result.metrics.total_patches = result.patches.len();
        result.metrics.max_depth_reached = context.depth;
        result.metrics.execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(result)
    }
    
    /// Resolve an action definition from the bundle
    fn resolve_action(&self, action_id: &str) -> Result<Value, String> {
        if let Some(actions) = self.bundle.actions.as_array() {
            actions.iter()
                .find(|a| {
                    // Check main ID
                    if a.get("id").and_then(|id| id.as_str()) == Some(action_id) {
                        return true;
                    }
                    
                    // Check aliases
                    if let Some(aliases) = a.get("aliases").and_then(|al| al.as_array()) {
                        aliases.iter().any(|alias| alias.as_str() == Some(action_id))
                    } else {
                        false
                    }
                })
                .cloned()
                .ok_or_else(|| format!("Action '{}' not found in bundle", action_id))
        } else {
            Err("Bundle has no actions array".to_string())
        }
    }
    
    /// Process template variables in action definition
    fn process_templates(&self, action: &Value, state: &Value, context: &ActionContext) -> Value {
        let mut processed = action.clone();
        
        // First apply standard template replacement
        processed = replace_template_vars(&processed, state);
        
        // Then apply actor template
        processed = replace_actor_template(&processed, &context.actor);
        
        // Apply custom template vars
        for (key, value) in &context.template_vars {
            processed = self.replace_custom_template(&processed, key, value);
        }
        
        // Special handling for {player} template
        processed = self.replace_custom_template(&processed, "player", &context.actor);
        
        // Replace {args.*} templates with values from context args
        processed = self.replace_args_templates(&processed, &context.args);
        
        processed
    }
    
    /// Replace {args.*} template variables
    fn replace_args_templates(&self, value: &Value, args: &Value) -> Value {
        match value {
            Value::String(s) => {
                let mut result = s.clone();
                
                // Look for {args.something} patterns
                if result.contains("{args.") {
                    if let Value::Object(args_obj) = args {
                        for (key, val) in args_obj {
                            let template = format!("{{args.{}}}", key);
                            if result.contains(&template) {
                                match val {
                                    Value::String(str_val) => {
                                        result = result.replace(&template, str_val);
                                    }
                                    Value::Number(num) => {
                                        result = result.replace(&template, &num.to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                
                Value::String(result)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.replace_args_templates(v, args)).collect())
            }
            Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (k, v) in obj {
                    new_obj.insert(k.clone(), self.replace_args_templates(v, args));
                }
                Value::Object(new_obj)
            }
            _ => value.clone()
        }
    }
    
    /// Replace a custom template variable
    fn replace_custom_template(&self, value: &Value, key: &str, replacement: &str) -> Value {
        let template = format!("{{{}}}", key);
        match value {
            Value::String(s) => {
                Value::String(s.replace(&template, replacement))
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.replace_custom_template(v, key, replacement)).collect())
            }
            Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (k, v) in obj {
                    new_obj.insert(k.clone(), self.replace_custom_template(v, key, replacement));
                }
                Value::Object(new_obj)
            }
            _ => value.clone()
        }
    }
    
    /// Check action conditions
    fn check_conditions(&self, action: &Value, state: &Value, context: &ActionContext) -> Result<bool, String> {
        if let Some(conditions) = action.get("when").and_then(|w| w.as_array()) {
            for condition in conditions {
                if !conditions::evaluate_condition(condition, state, &context.args, &context.actor)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    
    /// Merge two sets of arguments
    fn merge_args(&self, base: &Value, overlay: &Value) -> Value {
        match (base, overlay) {
            (Value::Object(base_obj), Value::Object(overlay_obj)) => {
                let mut merged = base_obj.clone();
                for (k, v) in overlay_obj {
                    merged.insert(k.clone(), v.clone());
                }
                Value::Object(merged)
            }
            _ => overlay.clone()
        }
    }
    
    /// Execute a verb
    fn execute_verb(
        &self,
        verb: &str,
        state: &mut Value,
        args: &Value,
        context: &ActionContext,
    ) -> Result<ActionResult, String> {
        println!("[ActionExecutor] Executing verb '{}' with args: {:?}", verb, args);
        
        // Check if we have a registered executor
        if let Some(executor) = self.verb_registry.get(verb) {
            executor.execute(state, args, context, self)
        } else {
            // Fallback to legacy verb execution
            println!("[ActionExecutor] Using legacy verb execution for '{}'", verb);
            let patches = crate::engine::verbs::apply_verb(state, verb, args, &self.bundle)?;
            
            let mut result = ActionResult::empty();
            result.patches = patches;
            Ok(result)
        }
    }
    
    /// Process a trigger
    fn process_trigger(
        &self,
        trigger: &Value,
        state: &Value,
        context: &ActionContext,
        result: &mut ActionResult,
        index: usize,
    ) -> Result<(), String> {
        // TODO: Implement trigger processing
        // For now, triggers are not processed
        Ok(())
    }
}

/// Generate a unique execution ID
fn generate_execution_id() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let id: u64 = rng.gen();
    format!("exec_{:016x}", id)
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_action_context() {
        let ctx = ActionContext::new("p1".to_string(), json!({"test": true}));
        assert_eq!(ctx.actor, "p1");
        assert_eq!(ctx.depth, 0);
        
        let child = ctx.child();
        assert_eq!(child.actor, "p1");
        assert_eq!(child.depth, 1);
        assert!(child.parent_context.is_some());
    }
    
    #[test]
    fn test_action_result_merge() {
        let mut result1 = ActionResult::empty();
        result1.patches.push(json!({"op": "add", "path": "/test", "value": 1}));
        result1.metrics.total_actions = 1;
        
        let mut result2 = ActionResult::empty();
        result2.patches.push(json!({"op": "add", "path": "/test2", "value": 2}));
        result2.metrics.total_actions = 1;
        
        result1.merge(result2);
        assert_eq!(result1.patches.len(), 2);
        assert_eq!(result1.metrics.total_actions, 2);
    }
}