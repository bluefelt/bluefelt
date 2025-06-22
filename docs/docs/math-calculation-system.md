# Math and Calculation System

The math and calculation system enables games to perform arithmetic operations, evaluate expressions, and maintain numeric game state like scores, resources, and counters.

## Overview

The system provides:
- Mathematical expression evaluation
- Arithmetic verbs for state manipulation
- Numeric conditions for game logic
- Support for variables and functions

## Verbs

### calculate

Evaluates a mathematical expression and stores the result.

```yaml
- verb: calculate
  expression: "score * multiplier + bonus"
  target: "scores.p1.total"
  variables:
    score: "scores.p1.base"
    multiplier: "gameState.multiplier"
    bonus: 10
```

**Parameters:**
- `expression`: Mathematical expression to evaluate
- `target`: State path where result will be stored
- `variables`: Map of variable names to values (paths or numbers)

### increment

Adds a value to an existing numeric value.

```yaml
- verb: increment
  target: "scores.p1"
  amount: 5  # Optional, defaults to 1
```

### decrement

Subtracts a value from an existing numeric value.

```yaml
- verb: decrement
  target: "resources.{player}.gold"
  amount: 3
```

### multiply

Multiplies an existing numeric value.

```yaml
- verb: multiply
  target: "scores.p1"
  factor: 2
```

### compareValues

Compares two numeric values (for side effects, use conditions for branching).

```yaml
- verb: compareValues
  left: "scores.p1"
  right: "scores.p2"
  operator: ">"
  result: "p1Leading"  # Optional, stores boolean result
```

## Expression Syntax

The expression evaluator supports:

### Operators
- Addition: `+`
- Subtraction: `-`
- Multiplication: `*`
- Division: `/`
- Modulo: `%`
- Parentheses: `()`

### Functions
- `min(a, b, ...)` - Returns minimum value
- `max(a, b, ...)` - Returns maximum value
- `abs(x)` - Absolute value
- `floor(x)` - Round down
- `ceil(x)` - Round up
- `round(x)` - Round to nearest integer

### Examples

```yaml
# Simple arithmetic
expression: "2 + 3 * 4"  # = 14

# With variables
expression: "score * 2 + bonus"

# Using functions
expression: "max(p1Score, p2Score)"
expression: "floor(total / players)"

# Complex expressions
expression: "(base + bonus) * multiplier - penalty"
```

## Conditions

### numeric.compare

Compares a numeric value or expression against a target.

```yaml
conditions:
  - type: numeric.compare
    with:
      path: "scores.{player}"
      operator: ">="
      value: 100
```

With expressions:

```yaml
conditions:
  - type: numeric.compare
    with:
      expression: "gold + silver * 0.5"
      operator: ">"
      value: 50
      variables:
        gold: "resources.{player}.gold"
        silver: "resources.{player}.silver"
```

## Usage Examples

### Score Tracking

```yaml
# Initialize scores
- verb: setState
  path: "scores"
  value:
    p1: 0
    p2: 0

# Award points
- verb: increment
  target: "scores.{player}"
  amount: 10

# Calculate bonus
- verb: calculate
  expression: "base * (1 + combo * 0.1)"
  target: "scores.{player}"
  variables:
    base: "scores.{player}"
    combo: "combos.{player}"
```

### Resource Management

```yaml
# Check if player can afford something
conditions:
  - type: numeric.compare
    with:
      path: "resources.{player}.gold"
      operator: ">="
      value: 25

# Deduct cost
then:
  - verb: decrement
    target: "resources.{player}.gold"
    amount: 25
```

### Victory Points Calculation

```yaml
# Calculate final score
- verb: calculate
  expression: "territories * 5 + gold + monuments * 10"
  target: "finalScores.{player}"
  variables:
    territories: "counts.{player}.territories"
    gold: "resources.{player}.gold"
    monuments: "counts.{player}.monuments"

# Determine winner
- uses: conditionalAction
  with:
    condition:
      condition: numeric.compare
      with:
        path: "finalScores.p1"
        operator: ">"
        value: "finalScores.p2"
    then:
      - action: setState
        with:
          path: "winner"
          value: "p1"
```

### Progressive Costs

```yaml
# Cost increases with each purchase
- verb: calculate
  expression: "10 * (1.5 ^ owned)"
  target: "costs.building"
  variables:
    owned: "counts.{player}.buildings"
```

## Best Practices

1. **Initialize numeric values** - Always initialize counters and scores before using them
2. **Use descriptive paths** - Store calculations in meaningful state locations
3. **Validate inputs** - Check for division by zero or invalid operations
4. **Round appropriately** - Use floor/ceil/round for integer game values
5. **Document formulas** - Comment complex calculations in your game files

## Integration with Other Systems

The math system works seamlessly with:
- **Conditions**: Use `numeric.compare` for branching logic
- **State Management**: Store results with `setState` or direct calculation
- **View Zones**: Display calculated values in strategic views
- **Action Parameters**: Use calculations in action costs or effects

## Future Enhancements

Potential additions to the system:
- Random number generation with seeds
- Statistical functions (average, median, etc.)
- Trigonometric functions for grid calculations
- Custom function definitions
- Array/list operations (sum, count, etc.)