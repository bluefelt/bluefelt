//! Tests for the patch optimization system

use json_patch::{Patch, PatchOperation, AddOperation, ReplaceOperation, RemoveOperation};
use serde_json::{json, Value};
use bluefelt_core::engine::patch_optimizer::PatchOptimizer;

#[test]
fn test_merge_adjacent_operations() {
    let optimizer = PatchOptimizer::default();
    
    // Remove followed by Add should become Replace
    let patch = Patch(vec![
        PatchOperation::Remove(RemoveOperation {
            path: "/test".parse().unwrap(),
        }),
        PatchOperation::Add(AddOperation {
            path: "/test".parse().unwrap(),
            value: json!("new value"),
        }),
    ]);

    let optimized = optimizer.optimize(patch);
    assert_eq!(optimized.0.len(), 1);
    assert!(matches!(optimized.0[0], PatchOperation::Replace(_)));
}

#[test]
fn test_consolidate_multiple_operations() {
    let optimizer = PatchOptimizer::default();
    
    // Multiple replacements on same path should keep only the last
    let patch = Patch(vec![
        PatchOperation::Replace(ReplaceOperation {
            path: "/value".parse().unwrap(),
            value: json!("first"),
        }),
        PatchOperation::Replace(ReplaceOperation {
            path: "/value".parse().unwrap(),
            value: json!("second"),
        }),
        PatchOperation::Replace(ReplaceOperation {
            path: "/value".parse().unwrap(),
            value: json!("final"),
        }),
    ]);

    let optimized = optimizer.optimize(patch);
    assert_eq!(optimized.0.len(), 1);
    
    if let PatchOperation::Replace(op) = &optimized.0[0] {
        assert_eq!(op.value, json!("final"));
    } else {
        panic!("Expected Replace operation");
    }
}

#[test]
fn test_array_operation_optimization() {
    let optimizer = PatchOptimizer::default();
    
    // Many operations on array indices should be replaced with single array replacement
    let mut operations = vec![];
    for i in 0..15 {
        operations.push(PatchOperation::Replace(ReplaceOperation {
            path: format!("/items/{}", i).parse().unwrap(),
            value: json!(i),
        }));
    }
    
    let patch = Patch(operations);
    let optimized = optimizer.optimize(patch);
    
    // Should be consolidated to a single replace operation on the array
    assert!(optimized.0.len() < 15);
    
    // Find the array replacement
    let has_array_replace = optimized.0.iter().any(|op| {
        if let PatchOperation::Replace(replace_op) = op {
            replace_op.path.to_string() == "/items"
        } else {
            false
        }
    });
    assert!(has_array_replace);
}

#[test]
fn test_optimization_with_mixed_operations() {
    let optimizer = PatchOptimizer::default();
    
    let patch = Patch(vec![
        // These should be merged
        PatchOperation::Remove(RemoveOperation {
            path: "/old".parse().unwrap(),
        }),
        PatchOperation::Add(AddOperation {
            path: "/old".parse().unwrap(),
            value: json!("replaced"),
        }),
        // This should be kept separate
        PatchOperation::Add(AddOperation {
            path: "/new".parse().unwrap(),
            value: json!("added"),
        }),
        // These should be consolidated (only last kept)
        PatchOperation::Replace(ReplaceOperation {
            path: "/counter".parse().unwrap(),
            value: json!(1),
        }),
        PatchOperation::Replace(ReplaceOperation {
            path: "/counter".parse().unwrap(),
            value: json!(2),
        }),
    ]);
    
    let optimized = optimizer.optimize(patch);
    
    // Should have 3 operations: merged replace for /old, add for /new, replace for /counter
    assert_eq!(optimized.0.len(), 3);
}

#[test]
fn test_small_patch_not_optimized() {
    let optimizer = PatchOptimizer::default();
    
    // Single operation should not be modified
    let patch = Patch(vec![
        PatchOperation::Add(AddOperation {
            path: "/test".parse().unwrap(),
            value: json!("value"),
        }),
    ]);
    
    let optimized = optimizer.optimize(patch.clone());
    assert_eq!(optimized.0.len(), 1);
}

#[test]
fn test_large_state_optimization() {
    // Simulate patches for a large game state update
    let mut patches = vec![];
    
    // Add many patches that update array elements
    for i in 0..50 {
        patches.push(json!({
            "op": "replace",
            "path": format!("/zones/deck/items/{}", i),
            "value": {"entity": format!("card-{}", i)}
        }));
    }
    
    // The broadcast_game_update function should optimize these
    // when there are more than 10 patches
    assert!(patches.len() > 10);
    
    // Convert to PatchOperation objects
    let patch_ops: Vec<PatchOperation> = patches.iter()
        .filter_map(|p| serde_json::from_value(p.clone()).ok())
        .collect();
    
    assert!(patch_ops.len() > 10);
    
    // Optimize
    let optimizer = PatchOptimizer::default();
    let optimized = optimizer.optimize(Patch(patch_ops));
    
    // Should be significantly fewer operations
    assert!(optimized.0.len() < 50);
}