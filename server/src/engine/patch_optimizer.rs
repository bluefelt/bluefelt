//! Optimized patch generation for large game states

use json_patch::{Patch, PatchOperation};
use serde_json::{Value, Map};
use std::collections::HashSet;

/// Configuration for patch optimization
pub struct PatchOptimizer {
    /// Maximum number of operations before switching to full replacement
    pub max_operations: usize,
    /// Minimum size (in bytes) of value to consider for optimization
    pub min_value_size: usize,
    /// Whether to merge adjacent operations
    pub merge_adjacent: bool,
}

impl Default for PatchOptimizer {
    fn default() -> Self {
        Self {
            max_operations: 100,
            min_value_size: 1024, // 1KB
            merge_adjacent: true,
        }
    }
}

impl PatchOptimizer {
    /// Optimize a patch by reducing operations and improving efficiency
    pub fn optimize(&self, mut patch: Patch) -> Patch {
        if patch.0.len() <= 1 {
            return patch;
        }

        // Step 1: Merge adjacent operations
        if self.merge_adjacent {
            patch = self.merge_adjacent_ops(patch);
        }

        // Step 2: Replace multiple operations on same path with single operation
        patch = self.consolidate_operations(patch);

        // Step 3: Replace large arrays of operations with single replace
        patch = self.optimize_array_operations(patch);

        patch
    }

    /// Merge adjacent add/remove operations that could be combined
    fn merge_adjacent_ops(&self, patch: Patch) -> Patch {
        let mut optimized = Vec::new();
        let mut skip_next = false;

        for (i, op) in patch.0.iter().enumerate() {
            if skip_next {
                skip_next = false;
                continue;
            }

            // Check if we can merge with next operation
            if i + 1 < patch.0.len() {
                let next_op = &patch.0[i + 1];
                
                match (op, next_op) {
                    // Remove followed by Add at same path = Replace
                    (PatchOperation::Remove(remove_op), PatchOperation::Add(add_op)) 
                        if remove_op.path == add_op.path => {
                        optimized.push(PatchOperation::Replace(json_patch::ReplaceOperation {
                            path: add_op.path.clone(),
                            value: add_op.value.clone(),
                        }));
                        skip_next = true;
                        continue;
                    }
                    _ => {}
                }
            }

            optimized.push(op.clone());
        }

        Patch(optimized)
    }

    /// Consolidate multiple operations on the same path
    fn consolidate_operations(&self, patch: Patch) -> Patch {
        let mut path_ops: std::collections::HashMap<String, Vec<(usize, &PatchOperation)>> = 
            std::collections::HashMap::new();

        // Group operations by path
        for (idx, op) in patch.0.iter().enumerate() {
            let path = match op {
                PatchOperation::Add(op) => &op.path,
                PatchOperation::Remove(op) => &op.path,
                PatchOperation::Replace(op) => &op.path,
                PatchOperation::Move(op) => &op.from,
                PatchOperation::Copy(op) => &op.from,
                PatchOperation::Test(op) => &op.path,
            };
            
            path_ops.entry(path.to_string())
                .or_insert_with(Vec::new)
                .push((idx, op));
        }

        // If multiple operations on same path, keep only the last effective one
        let mut operations_to_keep: HashSet<usize> = HashSet::new();
        
        for (_, ops) in path_ops {
            if ops.len() == 1 {
                operations_to_keep.insert(ops[0].0);
            } else {
                // Multiple operations on same path - analyze what to keep
                let mut last_write_idx = None;
                
                for (idx, op) in ops.iter().rev() {
                    match op {
                        PatchOperation::Add(_) | 
                        PatchOperation::Replace(_) | 
                        PatchOperation::Remove(_) => {
                            // These are write operations - keep the last one
                            if last_write_idx.is_none() {
                                last_write_idx = Some(*idx);
                            }
                        }
                        PatchOperation::Test(_) => {
                            // Keep all test operations
                            operations_to_keep.insert(*idx);
                        }
                        _ => {
                            // Keep move/copy operations
                            operations_to_keep.insert(*idx);
                        }
                    }
                }
                
                if let Some(idx) = last_write_idx {
                    operations_to_keep.insert(idx);
                }
            }
        }

        // Build optimized patch with only kept operations
        let optimized: Vec<PatchOperation> = patch.0
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| operations_to_keep.contains(idx))
            .map(|(_, op)| op)
            .collect();

        Patch(optimized)
    }

    /// Optimize operations on arrays
    fn optimize_array_operations(&self, patch: Patch) -> Patch {
        // Count operations by array path prefix
        let mut array_op_counts: std::collections::HashMap<String, usize> = 
            std::collections::HashMap::new();

        for op in &patch.0 {
            let path_str = match op {
                PatchOperation::Add(op) => op.path.to_string(),
                PatchOperation::Remove(op) => op.path.to_string(),
                PatchOperation::Replace(op) => op.path.to_string(),
                _ => continue,
            };

            // Check if this is an array index operation
            if let Some(idx) = path_str.rfind('/') {
                let prefix = &path_str[..idx];
                let suffix = &path_str[idx + 1..];
                
                // Check if suffix is a number (array index)
                if suffix.parse::<usize>().is_ok() {
                    *array_op_counts.entry(prefix.to_string()).or_insert(0) += 1;
                }
            }
        }

        // If too many operations on same array, replace the whole array
        let arrays_to_replace: HashSet<String> = array_op_counts
            .into_iter()
            .filter(|(_, count)| *count > 10) // Threshold for array replacement
            .map(|(path, _)| path)
            .collect();

        if arrays_to_replace.is_empty() {
            return patch;
        }

        // Filter out individual array operations and collect array values
        let mut array_values: std::collections::HashMap<String, Vec<Option<Value>>> = 
            std::collections::HashMap::new();
        let mut filtered_ops = Vec::new();

        for op in patch.0 {
            let should_filter = match &op {
                PatchOperation::Add(add_op) => {
                    let path_str = add_op.path.to_string();
                    if let Some(idx) = path_str.rfind('/') {
                        let prefix = &path_str[..idx];
                        let suffix = &path_str[idx + 1..];
                        
                        if arrays_to_replace.contains(prefix) {
                            if let Ok(index) = suffix.parse::<usize>() {
                                let value = Some(add_op.value.clone());
                                
                                let array = array_values.entry(prefix.to_string())
                                    .or_insert_with(Vec::new);
                                
                                // Ensure array is large enough
                                while array.len() <= index {
                                    array.push(None);
                                }
                                array[index] = value;
                                
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                PatchOperation::Replace(replace_op) => {
                    let path_str = replace_op.path.to_string();
                    if let Some(idx) = path_str.rfind('/') {
                        let prefix = &path_str[..idx];
                        let suffix = &path_str[idx + 1..];
                        
                        if arrays_to_replace.contains(prefix) {
                            if let Ok(index) = suffix.parse::<usize>() {
                                let value = Some(replace_op.value.clone());
                                
                                let array = array_values.entry(prefix.to_string())
                                    .or_insert_with(Vec::new);
                                
                                // Ensure array is large enough
                                while array.len() <= index {
                                    array.push(None);
                                }
                                array[index] = value;
                                
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if !should_filter {
                filtered_ops.push(op);
            }
        }

        // Add array replacement operations
        for (path, values) in array_values {
            let array_value = Value::Array(
                values.into_iter()
                    .map(|v| v.unwrap_or(Value::Null))
                    .collect()
            );
            
            filtered_ops.push(PatchOperation::Replace(json_patch::ReplaceOperation {
                path: path.parse().unwrap(),
                value: array_value,
            }));
        }

        Patch(filtered_ops)
    }

    /// Calculate the approximate size of a JSON value
    pub fn estimate_size(value: &Value) -> usize {
        match value {
            Value::Null => 4,
            Value::Bool(_) => 5,
            Value::Number(n) => n.to_string().len(),
            Value::String(s) => s.len() + 2,
            Value::Array(arr) => {
                arr.iter().map(Self::estimate_size).sum::<usize>() + arr.len() + 2
            }
            Value::Object(obj) => {
                obj.iter()
                    .map(|(k, v)| k.len() + 2 + Self::estimate_size(v) + 1)
                    .sum::<usize>() + 2
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use json_patch::{AddOperation, RemoveOperation};

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
                value: Value::String("new value".to_string()),
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
            PatchOperation::Replace(json_patch::ReplaceOperation {
                path: "/value".parse().unwrap(),
                value: Value::String("first".to_string()),
            }),
            PatchOperation::Replace(json_patch::ReplaceOperation {
                path: "/value".parse().unwrap(),
                value: Value::String("second".to_string()),
            }),
            PatchOperation::Replace(json_patch::ReplaceOperation {
                path: "/value".parse().unwrap(),
                value: Value::String("final".to_string()),
            }),
        ]);

        let optimized = optimizer.optimize(patch);
        assert_eq!(optimized.0.len(), 1);
        
        if let PatchOperation::Replace(op) = &optimized.0[0] {
            assert_eq!(op.value, Value::String("final".to_string()));
        } else {
            panic!("Expected Replace operation");
        }
    }

    #[test]
    fn test_estimate_size() {
        assert_eq!(PatchOptimizer::estimate_size(&Value::Null), 4);
        assert_eq!(PatchOptimizer::estimate_size(&Value::Bool(true)), 5);
        assert_eq!(PatchOptimizer::estimate_size(&Value::String("test".to_string())), 6);
        
        let array = Value::Array(vec![Value::Null, Value::Bool(true)]);
        assert!(PatchOptimizer::estimate_size(&array) > 10);
    }
}