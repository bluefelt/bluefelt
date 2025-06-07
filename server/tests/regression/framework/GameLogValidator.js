/**
 * Game Log Validator
 * Validates that game logs contain expected messages and don't contain unwanted messages
 */
class GameLogValidator {
  constructor() {
    this.logs = [];
    this.expectations = [];
  }

  /**
   * Add a log message
   */
  addLog(message) {
    this.logs.push(message);
  }

  /**
   * Clear all logs
   */
  clearLogs() {
    this.logs = [];
    this.expectations = [];
  }

  /**
   * Expect a log message to exist
   */
  expectLog(pattern, description) {
    this.expectations.push({
      type: 'exists',
      pattern,
      description: description || `Expected log matching: ${pattern}`
    });
  }

  /**
   * Expect a log message to NOT exist
   */
  expectNoLog(pattern, description) {
    this.expectations.push({
      type: 'not_exists',
      pattern,
      description: description || `Expected NO log matching: ${pattern}`
    });
  }

  /**
   * Expect logs to appear in a specific order
   */
  expectLogOrder(patterns, description) {
    this.expectations.push({
      type: 'order',
      patterns,
      description: description || `Expected logs in order: ${patterns.join(' -> ')}`
    });
  }

  /**
   * Validate all expectations
   */
  validate() {
    const results = [];
    
    for (const expectation of this.expectations) {
      switch (expectation.type) {
        case 'exists':
          results.push(this.validateExists(expectation));
          break;
        case 'not_exists':
          results.push(this.validateNotExists(expectation));
          break;
        case 'order':
          results.push(this.validateOrder(expectation));
          break;
      }
    }
    
    return {
      passed: results.every(r => r.passed),
      results
    };
  }

  validateExists(expectation) {
    const found = this.logs.some(log => this.matchesPattern(log, expectation.pattern));
    return {
      passed: found,
      description: expectation.description,
      actual: found ? `Found matching log` : `No matching log found`,
      logs: this.logs
    };
  }

  validateNotExists(expectation) {
    const found = this.logs.some(log => this.matchesPattern(log, expectation.pattern));
    return {
      passed: !found,
      description: expectation.description,
      actual: found ? `Found unwanted log: ${this.logs.find(log => this.matchesPattern(log, expectation.pattern))}` : `No unwanted log found`,
      logs: this.logs
    };
  }

  validateOrder(expectation) {
    let lastIndex = -1;
    const foundPatterns = [];
    
    for (const pattern of expectation.patterns) {
      const index = this.logs.findIndex((log, i) => 
        i > lastIndex && this.matchesPattern(log, pattern)
      );
      
      if (index === -1) {
        return {
          passed: false,
          description: expectation.description,
          actual: `Pattern "${pattern}" not found after previous patterns`,
          foundPatterns,
          logs: this.logs
        };
      }
      
      foundPatterns.push({ pattern, index, log: this.logs[index] });
      lastIndex = index;
    }
    
    return {
      passed: true,
      description: expectation.description,
      actual: `All patterns found in correct order`,
      foundPatterns,
      logs: this.logs
    };
  }

  matchesPattern(log, pattern) {
    if (pattern instanceof RegExp) {
      return pattern.test(log);
    }
    return log.includes(pattern);
  }

  /**
   * Print validation results
   */
  printResults(results) {
    console.log('\\n📋 Game Log Validation Results:');
    
    for (const result of results.results) {
      if (result.passed) {
        console.log(`  ✅ ${result.description}`);
      } else {
        console.log(`  ❌ ${result.description}`);
        console.log(`     Actual: ${result.actual}`);
        if (result.logs && result.logs.length > 0) {
          console.log(`     Logs:`);
          result.logs.forEach((log, i) => {
            console.log(`       [${i}] ${log}`);
          });
        }
      }
    }
    
    return results.passed;
  }
}

module.exports = { GameLogValidator };