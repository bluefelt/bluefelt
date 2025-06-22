//! ConditionalAction verb executor that properly queues sub-actions

use super::{ActionContext, ActionExecutor, ActionResult, ActionSource, TriggeredAction, VerbExecutor};
use crate::conditions;
use serde_json::{json, Value};

/// Executor for conditionalAction verb
pub struct ConditionalActionExecutor;

impl VerbExecutor for ConditionalActionExecutor {
    fn execute(
        &self,
        state: &mut Value,
        args: &Value,
        context: &ActionContext,
        executor: &ActionExecutor,
    ) -> Result<ActionResult, String> {
        println!("[ConditionalActionExecutor] Evaluating condition");
        
        // Get the condition
        let condition = args.get("condition")
            .ok_or("conditionalAction missing 'condition'")?;
        
        // Evaluate the condition
        let condition_met = if let Some(conditions) = condition.as_array() {
            // Multiple conditions (AND logic)
            conditions.iter().all(|cond| {
                match conditions::evaluate_condition(cond, state, &context.args, &context.actor) {
                    Ok(result) => result,
                    Err(e) => {
                        println!("[ConditionalActionExecutor] Error evaluating condition: {}", e);
                        false
                    }
                }
            })
        } else {
            // Single condition
            match conditions::evaluate_condition(condition, state, &context.args, &context.actor) {
                Ok(result) => result,
                Err(e) => {
                    println!("[ConditionalActionExecutor] Error evaluating condition: {}", e);
                    false
                }
            }
        };
        
        println!("[ConditionalActionExecutor] Condition result: {}", condition_met);
        
        // Determine which actions to queue
        let actions_to_queue = if condition_met {
            args.get("then").or_else(|| args.get("ifTrue"))
        } else {
            args.get("else").or_else(|| args.get("ifFalse"))
        };
        
        // Queue the actions for execution
        let mut result = ActionResult::empty();
        
        if let Some(actions) = actions_to_queue.and_then(|a| a.as_array()) {
            println!("[ConditionalActionExecutor] Queueing {} actions", actions.len());
            
            for (i, action) in actions.iter().enumerate() {
                if let Some(action_id) = action.get("action").and_then(|a| a.as_str()) {
                    let action_args = action.get("with").cloned().unwrap_or(json!({}));
                    
                    println!("[ConditionalActionExecutor] Queueing action: {} with args: {:?}", 
                        action_id, action_args);
                    
                    result.triggered_actions.push(TriggeredAction {
                        action_id: action_id.to_string(),
                        context: context.child_with_args(action_args),
                        source: ActionSource::ConditionalAction,
                        priority: 50 - i as i32, // Lower priority than direct then actions
                    });
                }
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::bundle::Bundle;
    
    #[test]
    fn test_conditional_action_true() {
        let executor = ConditionalActionExecutor;
        let mut state = json!({
            "test": true
        });
        
        let args = json!({
            "condition": {
                "condition": "valueEquals",
                "path": "/test",
                "value": true
            },
            "then": [
                {"action": "action1"},
                {"action": "action2", "with": {"foo": "bar"}}
            ],
            "else": [
                {"action": "action3"}
            ]
        });
        
        let context = ActionContext::new("p1".to_string(), json!({}));
        let bundle = Arc::new(Bundle::default());
        let action_executor = ActionExecutor::new(bundle);
        
        let result = executor.execute(&mut state, &args, &context, &action_executor).unwrap();
        
        assert_eq!(result.triggered_actions.len(), 2);
        assert_eq!(result.triggered_actions[0].action_id, "action1");
        assert_eq!(result.triggered_actions[1].action_id, "action2");
    }
    
    #[test]
    fn test_conditional_action_false() {
        let executor = ConditionalActionExecutor;
        let mut state = json!({
            "test": false
        });
        
        let args = json!({
            "condition": {
                "condition": "valueEquals",
                "path": "/test",
                "value": true
            },
            "then": [
                {"action": "action1"}
            ],
            "else": [
                {"action": "action3"}
            ]
        });
        
        let context = ActionContext::new("p1".to_string(), json!({}));
        let bundle = Arc::new(Bundle::default());
        let action_executor = ActionExecutor::new(bundle);
        
        let result = executor.execute(&mut state, &args, &context, &action_executor).unwrap();
        
        assert_eq!(result.triggered_actions.len(), 1);
        assert_eq!(result.triggered_actions[0].action_id, "action3");
    }
}