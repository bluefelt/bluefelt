use serde_json::Value;

pub fn get_zone_ref<'a>(state: &'a Value, zone_path: &str) -> Result<&'a Value, String> {
    navigate_path(state, zone_path)
}

pub fn get_zone_mut<'a>(state: &'a mut Value, zone_path: &str) -> Result<&'a mut Value, String> {
    navigate_path_mut(state, zone_path)
}

pub fn get_cell_value(state: &Value, cell_path: &str) -> Result<Value, String> {
    let path_parts: Vec<&str> = cell_path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in &path_parts {
        current = navigate_to_next(current, part, cell_path)?;
    }
    
    Ok(current.clone())
}

pub fn set_cell_value(state: &mut Value, cell_path: &str, value: Value) -> Result<(), String> {
    let path_parts: Vec<&str> = cell_path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in &path_parts {
        current = navigate_to_next_mut(current, part, cell_path)?;
    }
    
    *current = value;
    Ok(())
}

pub fn set_value_at_path(state: &mut Value, path: &str, value: Value) -> Result<(), String> {
    set_cell_value(state, path, value)
}

fn navigate_path<'a>(state: &'a Value, path: &str) -> Result<&'a Value, String> {
    let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in path_parts {
        current = current.get(part)
            .ok_or_else(|| format!("Path not found: {}", path))?;
    }
    
    Ok(current)
}

fn navigate_path_mut<'a>(state: &'a mut Value, path: &str) -> Result<&'a mut Value, String> {
    let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in path_parts {
        current = current.get_mut(part)
            .ok_or_else(|| format!("Path not found: {}", path))?;
    }
    
    Ok(current)
}

fn navigate_to_next<'a>(
    current: &'a Value,
    part: &str,
    full_path: &str,
) -> Result<&'a Value, String> {
    if let Ok(index) = part.parse::<usize>() {
        // This is an array index
        if let Some(array) = current.as_array() {
            if index < array.len() {
                Ok(&array[index])
            } else {
                Err(format!("Array index {} out of bounds (length: {})", index, array.len()))
            }
        } else {
            Err(format!("Expected array for numeric index '{}'", part))
        }
    } else {
        // This is an object key
        current.get(part)
            .ok_or_else(|| format!("Path not found: '{}' in path '{}'", part, full_path))
    }
}

fn navigate_to_next_mut<'a>(
    current: &'a mut Value,
    part: &str,
    full_path: &str,
) -> Result<&'a mut Value, String> {
    if let Ok(index) = part.parse::<usize>() {
        // This is an array index
        if let Some(array) = current.as_array_mut() {
            if index < array.len() {
                Ok(&mut array[index])
            } else {
                Err(format!("Array index {} out of bounds (length: {})", index, array.len()))
            }
        } else {
            Err(format!("Expected array for numeric index '{}'", part))
        }
    } else {
        // This is an object key
        if let Some(obj) = current.as_object_mut() {
            obj.get_mut(part)
                .ok_or_else(|| format!("Path not found: '{}' in path '{}'", part, full_path))
        } else {
            Err(format!("Expected object for key '{}'", part))
        }
    }
}