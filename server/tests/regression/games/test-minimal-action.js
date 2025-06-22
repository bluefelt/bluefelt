const { GameTestFramework } = require('../framework/GameTestFramework.js');

class MinimalActionTest extends GameTestFramework {
  constructor() {
    super('tic-tac-toe');
  }

  async runMinimalTest() {
    console.log('Starting minimal action test...');

    // Create lobby and connect players
    await this.createLobby();
    await this.connectPlayers(['Alice', 'Bob']);
    
    // Start game
    await this.startGame();
    
    console.log('Game started, executing action...');
    
    // Execute one action and immediately exit
    console.log('About to execute action...');
    const result = await this.executeAction('p1', 'placeMark', {
      location: '/zones/board/cells/0/0',
      entity: 'mark_p1'
    });
    console.log('Action method returned:', result);
    
    console.log('✅ Action completed successfully!');
    process.exit(0);
  }
}

async function runTest() {
  const test = new MinimalActionTest();
  await test.runMinimalTest();
}

runTest().catch(console.error);