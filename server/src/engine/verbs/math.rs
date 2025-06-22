use serde_json::{json, Value};
use std::collections::HashMap;

/// Evaluates a mathematical expression with support for variables
pub fn evaluate_expression(expr: &str, variables: &HashMap<String, f64>) -> Result<f64, String> {
    // This is a simple expression evaluator that supports:
    // - Basic arithmetic: +, -, *, /, %
    // - Parentheses: ()
    // - Variables: referenced by name
    // - Functions: min(), max(), abs(), floor(), ceil()
    
    let expr = expr.trim();
    
    // If it's just a number, return it
    if let Ok(num) = expr.parse::<f64>() {
        return Ok(num);
    }
    
    // If it's a variable, look it up
    if let Some(value) = variables.get(expr) {
        return Ok(*value);
    }
    
    // For now, implement a simple recursive descent parser
    // This can be enhanced later with a proper expression parser library
    parse_expression(expr, variables)
}

fn parse_expression(expr: &str, variables: &HashMap<String, f64>) -> Result<f64, String> {
    let expr = expr.trim();
    
    // Handle parentheses
    if expr.starts_with('(') && expr.ends_with(')') {
        // Need to check if these are the outermost matching parentheses
        let mut depth = 0;
        let mut is_outermost = true;
        for (i, ch) in expr.chars().enumerate() {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 && i < expr.len() - 1 {
                    is_outermost = false;
                    break;
                }
            }
        }
        if is_outermost {
            return parse_expression(&expr[1..expr.len()-1], variables);
        }
    }
    
    // Handle functions (but make sure it's not just a parenthesized expression)
    if let Some(func_end) = expr.find('(') {
        let func_name = expr[..func_end].trim();
        
        // Check if this is actually a function call (has a name before the parenthesis)
        if !func_name.is_empty() && func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            let args_end = expr.rfind(')').ok_or("Missing closing parenthesis")?;
            let args_str = &expr[func_end+1..args_end];
            
            match func_name {
            "min" => {
                let args = parse_comma_separated(args_str, variables)?;
                args.into_iter().reduce(f64::min).ok_or("min() requires at least one argument".to_string())
            }
            "max" => {
                let args = parse_comma_separated(args_str, variables)?;
                args.into_iter().reduce(f64::max).ok_or("max() requires at least one argument".to_string())
            }
            "abs" => {
                let val = parse_expression(args_str, variables)?;
                Ok(val.abs())
            }
            "floor" => {
                let val = parse_expression(args_str, variables)?;
                Ok(val.floor())
            }
            "ceil" => {
                let val = parse_expression(args_str, variables)?;
                Ok(val.ceil())
            }
            "round" => {
                let val = parse_expression(args_str, variables)?;
                Ok(val.round())
            }
            _ => Err(format!("Unknown function: {}", func_name))
            }
        } else {
            // Not a function call, handle as normal expression
            handle_binary_operations(expr, variables)
        }
    } else {
        // No parentheses, handle as normal expression
        handle_binary_operations(expr, variables)
    }
}

fn handle_binary_operations(expr: &str, variables: &HashMap<String, f64>) -> Result<f64, String> {
        let expr = expr.trim();
        
        // Handle binary operations (left-to-right associativity)
        // Order of operations: *, /, % before +, -
        
        // First try addition/subtraction (lowest precedence)
        if let Some(pos) = find_operator(expr, &['+', '-']) {
            let (left, op, right) = split_at_operator(expr, pos)?;
            let left_val = parse_expression(left, variables)?;
            let right_val = parse_expression(right, variables)?;
            
            match op {
                '+' => Ok(left_val + right_val),
                '-' => Ok(left_val - right_val),
                _ => unreachable!()
            }
        }
        // Then try multiplication/division/modulo (higher precedence)
        else if let Some(pos) = find_operator(expr, &['*', '/', '%']) {
            let (left, op, right) = split_at_operator(expr, pos)?;
            let left_val = parse_expression(left, variables)?;
            let right_val = parse_expression(right, variables)?;
            
            match op {
                '*' => Ok(left_val * right_val),
                '/' => {
                    if right_val == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(left_val / right_val)
                    }
                }
                '%' => Ok(left_val % right_val),
                _ => unreachable!()
            }
        }
        // Must be a variable, number, or unary minus
        else if let Ok(num) = expr.parse::<f64>() {
            Ok(num)
        } else if let Some(value) = variables.get(expr) {
            Ok(*value)
        } else if expr.starts_with('-') && expr.len() > 1 {
            // Handle unary minus
            let inner = &expr[1..];
            parse_expression(inner, variables).map(|v| -v)
        } else {
            Err(format!("Unknown variable or invalid expression: {}", expr))
        }
    }

fn find_operator(expr: &str, operators: &[char]) -> Option<usize> {
    let mut depth = 0;
    let chars: Vec<char> = expr.chars().collect();
    
    // Search from right to left for left-to-right associativity
    for i in (0..chars.len()).rev() {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if depth == 0 && operators.contains(&c) => {
                // Don't treat minus at the beginning as a binary operator (it's a unary minus)
                if c == '-' && i == 0 {
                    continue;
                }
                // Also check if this minus follows another operator (making it unary)
                if c == '-' && i > 0 {
                    let prev_char = chars[i-1];
                    if prev_char == '+' || prev_char == '-' || prev_char == '*' || prev_char == '/' || prev_char == '%' || prev_char == '(' {
                        continue;
                    }
                }
                return Some(i);
            }
            _ => {}
        }
    }
    None
}

fn split_at_operator(expr: &str, pos: usize) -> Result<(&str, char, &str), String> {
    let chars: Vec<char> = expr.chars().collect();
    if pos >= chars.len() {
        return Err("Invalid operator position".to_string());
    }
    
    let left = expr[..pos].trim();
    let right = expr[pos+1..].trim();
    let op = chars[pos];
    
    if left.is_empty() || right.is_empty() {
        return Err("Invalid expression: empty operand".to_string());
    }
    
    Ok((left, op, right))
}

fn parse_comma_separated(args: &str, variables: &HashMap<String, f64>) -> Result<Vec<f64>, String> {
    let mut results = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    
    for ch in args.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                results.push(parse_expression(&current, variables)?);
                current.clear();
            }
            _ => current.push(ch)
        }
    }
    
    if !current.trim().is_empty() {
        results.push(parse_expression(&current, variables)?);
    }
    
    Ok(results)
}

/// Extracts numeric value from a path in the game state
pub fn get_numeric_value(state: &Value, path: &str) -> Result<f64, String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = state;
    
    for part in parts {
        current = current.get(part)
            .ok_or_else(|| format!("Path not found: {}", path))?;
    }
    
    current.as_f64()
        .or_else(|| current.as_i64().map(|i| i as f64))
        .or_else(|| current.as_u64().map(|u| u as f64))
        .ok_or_else(|| format!("Value at {} is not numeric", path))
}

/// Sets a numeric value at a path in the game state
pub fn set_numeric_value(state: &mut Value, path: &str, value: f64) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = state;
    
    // Navigate to the parent of the target
    for i in 0..parts.len() - 1 {
        // Create intermediate objects if they don't exist
        if !current.get(parts[i]).is_some() {
            current[parts[i]] = json!({});
        }
        current = current.get_mut(parts[i])
            .ok_or_else(|| format!("Cannot navigate to path: {}", path))?;
    }
    
    // Set the final value
    let last_part = parts.last().ok_or("Empty path")?;
    current[last_part] = json!(value);
    
    Ok(())
}

/// Calculate verb: evaluates an expression and stores the result
pub fn apply_calculate(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let expression = args["expression"].as_str()
        .ok_or("Missing expression parameter")?;
    
    let target = args["target"].as_str()
        .ok_or("Missing target parameter")?;
    
    // Build variables map from current state
    let mut variables = HashMap::new();
    
    // Add any explicit variables from args
    if let Some(vars) = args.get("variables").and_then(|v| v.as_object()) {
        for (key, val) in vars {
            if let Some(path) = val.as_str() {
                // It's a path reference
                let value = get_numeric_value(state, path)?;
                variables.insert(key.clone(), value);
            } else if let Some(num) = val.as_f64() {
                // It's a direct numeric value
                variables.insert(key.clone(), num);
            }
        }
    }
    
    // Evaluate the expression
    let result = evaluate_expression(expression, &variables)?;
    
    // Store the result
    set_numeric_value(state, target, result)?;
    
    Ok(vec![])
}

/// Increment verb: adds a value to an existing numeric value
pub fn apply_increment(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let target = args["target"].as_str()
        .ok_or("Missing target parameter")?;
    
    let amount = args["amount"].as_f64()
        .or_else(|| args["amount"].as_i64().map(|i| i as f64))
        .unwrap_or(1.0);
    
    let current = get_numeric_value(state, target).unwrap_or(0.0);
    set_numeric_value(state, target, current + amount)?;
    
    Ok(vec![])
}

/// Decrement verb: subtracts a value from an existing numeric value
pub fn apply_decrement(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let target = args["target"].as_str()
        .ok_or("Missing target parameter")?;
    
    let amount = args["amount"].as_f64()
        .or_else(|| args["amount"].as_i64().map(|i| i as f64))
        .unwrap_or(1.0);
    
    let current = get_numeric_value(state, target).unwrap_or(0.0);
    set_numeric_value(state, target, current - amount)?;
    
    Ok(vec![])
}

/// Multiply verb: multiplies an existing numeric value
pub fn apply_multiply(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let target = args["target"].as_str()
        .ok_or("Missing target parameter")?;
    
    let factor = args["factor"].as_f64()
        .or_else(|| args["factor"].as_i64().map(|i| i as f64))
        .ok_or("Missing factor parameter")?;
    
    let current = get_numeric_value(state, target).unwrap_or(0.0);
    set_numeric_value(state, target, current * factor)?;
    
    Ok(vec![])
}

/// Compare verb: compares two values and conditionally executes actions
pub fn apply_compare_values(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let left_val = if let Some(left_path) = args["left"].as_str() {
        get_numeric_value(state, left_path)?
    } else if let Some(num) = args["left"].as_f64() {
        num
    } else {
        return Err("Invalid left value".to_string());
    };
    
    let right_val = if let Some(right_path) = args["right"].as_str() {
        get_numeric_value(state, right_path)?
    } else if let Some(num) = args["right"].as_f64() {
        num
    } else {
        return Err("Invalid right value".to_string());
    };
    
    let operator = args["operator"].as_str()
        .ok_or("Missing operator parameter")?;
    
    let condition_met = match operator {
        "==" => left_val == right_val,
        "!=" => left_val != right_val,
        ">" => left_val > right_val,
        "<" => left_val < right_val,
        ">=" => left_val >= right_val,
        "<=" => left_val <= right_val,
        _ => return Err(format!("Unknown operator: {}", operator))
    };
    
    // Store the comparison result if requested
    if let Some(result_path) = args["result"].as_str() {
        state[result_path] = json!(condition_met);
    }
    
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_evaluate_expression() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 10.0);
        vars.insert("y".to_string(), 5.0);
        
        assert_eq!(evaluate_expression("2 + 3", &vars).unwrap(), 5.0);
        assert_eq!(evaluate_expression("10 - 4", &vars).unwrap(), 6.0);
        assert_eq!(evaluate_expression("3 * 4", &vars).unwrap(), 12.0);
        assert_eq!(evaluate_expression("20 / 4", &vars).unwrap(), 5.0);
        assert_eq!(evaluate_expression("10 % 3", &vars).unwrap(), 1.0);
        
        assert_eq!(evaluate_expression("x + y", &vars).unwrap(), 15.0);
        assert_eq!(evaluate_expression("x * 2 + y", &vars).unwrap(), 25.0);
        assert_eq!(evaluate_expression("(x + y) * 2", &vars).unwrap(), 30.0);
        
        assert_eq!(evaluate_expression("max(x, y)", &vars).unwrap(), 10.0);
        assert_eq!(evaluate_expression("min(x, y)", &vars).unwrap(), 5.0);
        assert_eq!(evaluate_expression("abs(-5)", &vars).unwrap(), 5.0);
        assert_eq!(evaluate_expression("floor(3.7)", &vars).unwrap(), 3.0);
        assert_eq!(evaluate_expression("ceil(3.2)", &vars).unwrap(), 4.0);
        assert_eq!(evaluate_expression("round(3.6)", &vars).unwrap(), 4.0);
    }
    
    #[test]
    fn test_calculate_verb() {
        let mut state = json!({
            "scores": {
                "p1": 10,
                "p2": 15
            },
            "multiplier": 2
        });
        
        // Calculate p1's score times multiplier
        let args = json!({
            "expression": "score * mult",
            "target": "scores.p1_adjusted",
            "variables": {
                "score": "scores.p1",
                "mult": "multiplier"
            }
        });
        
        apply_calculate(&mut state, &args).unwrap();
        assert_eq!(state["scores"]["p1_adjusted"], 20.0);
        
        // Calculate sum of scores
        let args = json!({
            "expression": "p1 + p2",
            "target": "scores.total",
            "variables": {
                "p1": "scores.p1",
                "p2": "scores.p2"
            }
        });
        
        apply_calculate(&mut state, &args).unwrap();
        assert_eq!(state["scores"]["total"], 25.0);
    }
    
    #[test]
    fn test_increment_decrement() {
        let mut state = json!({
            "counter": 5
        });
        
        apply_increment(&mut state, &json!({"target": "counter", "amount": 3})).unwrap();
        assert_eq!(state["counter"], 8.0);
        
        apply_decrement(&mut state, &json!({"target": "counter", "amount": 2})).unwrap();
        assert_eq!(state["counter"], 6.0);
        
        apply_multiply(&mut state, &json!({"target": "counter", "factor": 2})).unwrap();
        assert_eq!(state["counter"], 12.0);
    }
}