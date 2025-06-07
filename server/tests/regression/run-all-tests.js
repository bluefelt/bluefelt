#!/usr/bin/env node

/**
 * Master test runner for all game regression tests
 * Runs each game's test suite and reports overall results
 */

const { spawn } = require('child_process');
const path = require('path');

const gameTests = [
  { name: 'Tic-Tac-Toe', file: 'test-tic-tac-toe.js' },
  { name: 'Connect Four', file: 'test-connect-four.js' },
  { name: 'Three Men\'s Morris', file: 'test-three-mens-morris.js' },
  { name: 'Go Fish', file: 'test-go-fish.js' }
];

async function runTest(testFile, testName) {
  return new Promise((resolve) => {
    console.log(`\n${'='.repeat(60)}`);
    console.log(`Running ${testName} Tests`);
    console.log(`${'='.repeat(60)}\n`);
    
    const testPath = path.join(__dirname, 'games', testFile);
    const child = spawn('node', [testPath], {
      stdio: 'inherit',
      cwd: __dirname
    });
    
    child.on('close', (code) => {
      resolve(code === 0);
    });
    
    child.on('error', (err) => {
      console.error(`Failed to run ${testName} test:`, err);
      resolve(false);
    });
  });
}

async function runAllTests() {
  console.log(`
╔═══════════════════════════════════════════════════════════════╗
║           BLUEFELT GAME REGRESSION TEST SUITE                 ║
║                                                               ║
║  Testing: Connect Four, Three Men's Morris, Go Fish           ║
║           and Tic-Tac-Toe                                     ║
╚═══════════════════════════════════════════════════════════════╝
`);

  console.log('⚠️  Make sure the Bluefelt server is running on port 8000!\n');
  console.log('Start with: cd server && cargo run\n');
  
  const results = [];
  
  for (const test of gameTests) {
    const success = await runTest(test.file, test.name);
    results.push({ name: test.name, success });
  }
  
  // Summary
  console.log(`\n${'='.repeat(60)}`);
  console.log('TEST SUMMARY');
  console.log(`${'='.repeat(60)}\n`);
  
  let allPassed = true;
  results.forEach(result => {
    const status = result.success ? '✅ PASSED' : '❌ FAILED';
    console.log(`${result.name.padEnd(20)} ${status}`);
    if (!result.success) allPassed = false;
  });
  
  console.log(`\n${'='.repeat(60)}`);
  
  if (allPassed) {
    console.log('✅ ALL TESTS PASSED!');
  } else {
    console.log('❌ SOME TESTS FAILED!');
  }
  
  console.log(`${'='.repeat(60)}\n`);
  
  process.exit(allPassed ? 0 : 1);
}

// Run tests
runAllTests().catch(err => {
  console.error('Test runner failed:', err);
  process.exit(1);
});