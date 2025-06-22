use bluefelt_core::bundle::Bundle;
use bluefelt_core::engine::action_executor::{
    ActionExecutor, ActionContext, ActionResult, ActionSource
};
use serde_json::{json, Value};
use std::sync::Arc;

/// Test helper to create a minimal bundle with test actions
fn create_test_bundle() -> Bundle {
    let manifest = json!({
        "id": "test-game",
        "version": "1.0.0",
        "metadata": {
            "name": "Test Game",
            "description": "A test game for ActionExecutor tests",
            "players": {"min": 2, "max": 4}
        }
    });

    Bundle {
        manifest,
        actions: json!([
            {
                "id": "simpleAction",
                "uses": "setState",
                "with": {
                    "path": "/testValue",
                    "value": "executed"
                }
            },
            {
                "id": "aliasAction",
                "uses": "simpleAction"
            },
            {
                "id": "templateAction",
                "uses": "setState",
                "with": {
                    "path": "/player",
                    "value": "{player}"
                }
            },
            {
                "id": "argsTemplateAction",
                "uses": "setState",
                "with": {
                    "path": "/target",
                    "value": "{args.location}"
                }
            },
            {
                "id": "conditionalAction",
                "uses": "conditionalAction",
                "with": {
                    "condition": {
                        "condition": "zone.isEmpty",
                        "with": {
                            "zone": "/zones/test"
                        }
                    },
                    "then": [
                        {"action": "simpleAction"}
                    ]
                }
            },
            {
                "id": "nestedThenAction",
                "uses": "setState",
                "with": {
                    "path": "/executed",
                    "value": "first"
                },
                "then": [
                    {
                        "action": "setState",
                        "with": {
                            "path": "/executed",
                            "value": "second"
                        }
                    }
                ]
            },
            {
                "id": "recursiveAction",
                "uses": "setState", 
                "with": {
                    "path": "/count",
                    "value": 1
                },
                "then": [
                    {"action": "recursiveAction"}
                ]
            }
        ]),
        entities: json!({}),
        phases: json!({}),
        zones: json!({})
    }
}

/// Test helper to create initial game state
fn create_test_state() -> Value {
    json!({
        "currentPlayer": "p1",
        "players": ["Alice", "Bob"],
        "zones": {
            "test": null
        },
        "testValue": null,
        "executed": null,
        "player": null,
        "target": null,
        "count": 0
    })
}

#[test]
fn test_simple_action_execution() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "simpleAction", context).unwrap();
    
    assert_eq!(state["testValue"], "executed");
    assert!(result.patches.len() > 0);
    assert!(result.triggered_actions.is_empty());
}

#[test]
fn test_action_alias_resolution() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "aliasAction", context).unwrap();
    
    // aliasAction should execute simpleAction's behavior
    assert_eq!(state["testValue"], "executed");
    assert!(result.patches.len() > 0);
}

#[test]
fn test_template_variable_replacement() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "templateAction", context).unwrap();
    
    assert_eq!(state["player"], "p1");
    assert!(result.patches.len() > 0);
}

#[test]
fn test_args_template_replacement() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    let args = json!({"location": "/zones/board/cells/0/0"});
    let context = ActionContext::new("p1".to_string(), args);
    
    let result = executor.execute_action(&mut state, "argsTemplateAction", context).unwrap();
    
    assert_eq!(state["target"], "/zones/board/cells/0/0");
    assert!(result.patches.len() > 0);
}

#[test]
fn test_then_actions_execution() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "nestedThenAction", context).unwrap();
    
    // First action should execute immediately
    assert_eq!(state["executed"], "first");
    
    // Should have one triggered then action
    assert_eq!(result.triggered_actions.len(), 1);
    assert_eq!(result.triggered_actions[0].source, ActionSource::ThenAction);
    
    // Execute the triggered action
    let triggered_action = &result.triggered_actions[0];
    executor.execute_action(&mut state, &triggered_action.action_id, triggered_action.context.clone()).unwrap();
    
    // Now should be updated to second value
    assert_eq!(state["executed"], "second");
}

#[test]
fn test_conditional_action_with_false_condition() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    // Set up state so condition is false (zone is not empty)
    state["zones"]["test"] = json!("something");
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "conditionalAction", context).unwrap();
    
    // Condition is false, so testValue should not be set
    assert_eq!(state["testValue"], Value::Null);
    assert!(result.triggered_actions.is_empty());
}

#[test]
fn test_conditional_action_with_true_condition() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    // Set up state so condition is true (zone is empty/null)
    state["zones"]["test"] = Value::Null;
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "conditionalAction", context).unwrap();
    
    // Condition is true, so should have triggered the then action
    assert_eq!(result.triggered_actions.len(), 1);
    
    // Execute the triggered action
    let triggered_action = &result.triggered_actions[0];
    executor.execute_action(&mut state, &triggered_action.action_id, triggered_action.context.clone()).unwrap();
    
    // Now testValue should be set
    assert_eq!(state["testValue"], "executed");
}

#[test]
fn test_depth_limit_prevents_infinite_recursion() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone()).with_max_depth(5); // Small limit for testing
    let mut state = create_test_state();
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    // This should not panic or run forever due to depth limits
    let result = executor.execute_action(&mut state, "recursiveAction", context);
    
    // Should succeed initially but prevent deep recursion
    assert!(result.is_ok());
    
    // Should have set count to 1 from initial execution
    assert_eq!(state["count"], 1);
    
    // Should have triggered a then action
    let result = result.unwrap();
    assert_eq!(result.triggered_actions.len(), 1);
    
    // Try to execute several levels of recursion and verify depth limit is hit
    let mut next_context = result.triggered_actions[0].context.clone();
    for i in 1..=10 {
        if next_context.depth > executor.max_depth {
            // Should return error when depth exceeded
            let recursive_result = executor.execute_action(&mut state, "recursiveAction", next_context);
            assert!(recursive_result.is_err());
            break;
        }
        
        let recursive_result = executor.execute_action(&mut state, "recursiveAction", next_context.clone()).unwrap();
        if recursive_result.triggered_actions.is_empty() {
            break;
        }
        next_context = recursive_result.triggered_actions[0].context.clone();
        
        if i == 10 {
            panic!("Should have hit depth limit before 10 iterations");
        }
    }
}

#[test]
fn test_action_priority_ordering() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "nestedThenAction", context).unwrap();
    
    // Should have triggered actions with correct source
    assert_eq!(result.triggered_actions.len(), 1);
    assert_eq!(result.triggered_actions[0].source, ActionSource::ThenAction);
    assert!(result.triggered_actions[0].priority > 0); // Should have higher priority
}

#[test]
fn test_missing_action_error() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    let context = ActionContext::new("p1".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "nonExistentAction", context);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found") || result.unwrap_err().contains("missing"));
}

#[test]
fn test_context_propagation() {
    let bundle = Arc::new(create_test_bundle());
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = create_test_state();
    
    let context = ActionContext::new("p2".to_string(), json!({}));
    
    let result = executor.execute_action(&mut state, "templateAction", context).unwrap();
    
    // Should use the context actor for template replacement
    assert_eq!(state["player"], "p2");
}

#[test]
fn test_multiple_template_replacements() {
    let custom_bundle = Bundle {
        manifest: json!({
            "id": "test-game",
            "version": "1.0.0",
            "metadata": {
                "name": "Test Game",
                "description": "A test game",
                "players": {"min": 2, "max": 4}
            }
        }),
        actions: json!([
            {
                "id": "multiTemplateAction",
                "uses": "setState",
                "with": {
                    "path": "/result",
                    "value": "actor={actor},target={args.target},value={args.value}"
                }
            }
        ]),
        entities: json!({}),
        phases: json!({}),
        zones: json!({})
    };
    
    let bundle = Arc::new(custom_bundle);
    let executor = ActionExecutor::new(bundle.clone());
    let mut state = json!({
        "result": null
    });
    
    let args = json!({
        "target": "cell_0_0",
        "value": 42
    });
    let context = ActionContext::new("p2".to_string(), args);
    
    let result = executor.execute_action(&mut state, "multiTemplateAction", context).unwrap();
    
    assert_eq!(state["result"], "actor=p2,target=cell_0_0,value=42");
}