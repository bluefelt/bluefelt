const { GameTestFramework } = require('../framework/GameTestFramework.js');

class SimpleActionTest extends GameTestFramework {
  constructor() {
    super('tic-tac-toe');
  }

  async runTest() {
    console.log('\n============================================================');
    console.log('Testing Simple Action Execution');
    console.log('============================================================\n');

    try {
      // Create lobby and connect players
      await this.createLobby();
      await this.connectPlayers(['Alice', 'Bob']);
      
      // Start game
      await this.startGame();
      
      console.log('\n🎯 Testing Simple Action');
      
      // Check initial action map
      if (this.actionMap && this.actionMap.p1) {
        const initialActions = Object.keys(this.actionMap.p1).length;
        console.log(`  Initial actions for p1: ${initialActions}`);
      } else {
        console.log('  ❌ No action map available');
        return;
      }
      
      // Execute one action
      console.log('\n  Executing placeMark action...');
      await this.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/0/0',
        entity: 'mark_p1'
      });
      
      // No need to wait - the executeAction method waits for the response
      
      // Check updated action map
      if (this.actionMap && this.actionMap.p2) {
        const p2Actions = Object.keys(this.actionMap.p2).length;
        console.log(`  Actions for p2 after move: ${p2Actions}`);
        console.log('  ✅ Action executed successfully!');
      } else {
        console.log('  ❌ Action map not updated properly');
      }
      
      console.log('\n✅ Test completed successfully!');
      
    } catch (error) {
      console.error('❌ Test failed:', error.message);
    } finally {
      await this.cleanup();
    }
  }
}

async function runTest() {
  const test = new SimpleActionTest();
  await test.runTest();
}

if (require.main === module) {
  runTest().catch(console.error);
}

module.exports = { SimpleActionTest };