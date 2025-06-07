// Example WebSocket regression test template
// This demonstrates the structure for game-specific E2E tests

const { GameTestFramework } = require('../framework/GameTestFramework.js');
const assert = require('assert');

class ExampleGameTest extends GameTestFramework {
  constructor() {
    super('example-game'); // Replace with your game ID
  }

  async runAllTests() {
    console.log('🎮 Running Example Game Tests\n');
    
    try {
      // Test categories - organize your tests logically
      await this.testGameSetup();
      await this.testBasicGameplay();
      await this.testWinConditions();
      await this.testDrawConditions();
      await this.testInvalidActions();
      await this.testEdgeCases();
      await this.testConcurrency();
      
      console.log('\n✅ All Example Game tests passed!');
    } catch (error) {
      console.error('\n❌ Test suite failed:', error.message);
      process.exit(1);
    }
  }

  // Category 1: Game Setup Tests
  async testGameSetup() {
    console.log('📋 Testing Game Setup...');
    
    await this.testScenario('Game initializes with correct state', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      
      // Verify initial state
      this.assert.strictEqual(this.gameStarted, false, 'Game should not auto-start');
      
      await this.startGame(lobby.id);
      
      // Verify game started correctly
      this.assert.strictEqual(this.gameStarted, true, 'Game should be started');
      this.assert.strictEqual(this.currentPlayer, 'Alice', 'First player should be Alice');
      this.assert.strictEqual(this.phase, 'play', 'Should be in play phase');
    });
    
    await this.testScenario('Handles variable player counts', async () => {
      const lobby = await this.createLobby();
      
      // Test with 3 players if supported
      await this.connectPlayers(lobby.id, ['Alice', 'Bob', 'Charlie']);
      await this.startGame(lobby.id);
      
      // Verify all players are in the game
      const players = this.getState('/game/players');
      this.assert.strictEqual(players.length, 3, 'Should have 3 players');
    });
  }

  // Category 2: Basic Gameplay Tests
  async testBasicGameplay() {
    console.log('\n🎲 Testing Basic Gameplay...');
    
    await this.testScenario('Players can make valid moves with proper logs', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Set up log expectations
      this.expectLog('Alice placed a piece', 'Alice placement log');
      this.expectLog('Bob placed a piece', 'Bob placement log');
      this.expectNoLog('Turn passes to', 'No unnecessary turn logs');
      this.expectNoLog('{', 'No unresolved template variables');
      this.expectLogOrder([
        'Alice placed a piece',
        'Bob placed a piece'
      ], 'Moves logged in correct order');
      
      // Alice makes first move
      await this.executeAction('Alice', 'place', {
        location: '/zones/board/0/0'
      });
      
      // Verify move was made
      const cell = this.getState('/zones/board/0/0');
      this.assert.strictEqual(cell.entity, 'piece_Alice', 'Alice\'s piece should be placed');
      
      // Verify turn switched
      this.assert.strictEqual(this.currentPlayer, 'Bob', 'Turn should switch to Bob');
      
      // Bob makes move
      await this.executeAction('Bob', 'place', {
        location: '/zones/board/1/1'
      });
      
      // Verify Bob's move
      const bobCell = this.getState('/zones/board/1/1');
      this.assert.strictEqual(bobCell.entity, 'piece_Bob', 'Bob\'s piece should be placed');
      
      // Wait for logs to be processed
      await this.wait(500);
      
      // Validate all log expectations
      const logsValid = this.validateLogs();
      assert(logsValid, 'Game logs should match all expectations');
    });
    
    await this.testScenario('Action map updates correctly', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Check initial action map
      const initialActions = this.getState('/ui/actionMap');
      this.assert.ok(initialActions['/zones/board/0/0'], 'Cell 0,0 should be actionable');
      
      // Make a move
      await this.executeAction('Alice', 'place', {
        location: '/zones/board/0/0'
      });
      
      // Verify action map updated
      const updatedActions = this.getState('/ui/actionMap');
      this.assert.strictEqual(
        updatedActions['/zones/board/0/0'], 
        undefined, 
        'Cell 0,0 should no longer be actionable'
      );
    });
  }

  // Category 3: Win Conditions
  async testWinConditions() {
    console.log('\n🏆 Testing Win Conditions...');
    
    await this.testScenario('Horizontal win detected with proper win log', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Expect win announcement
      this.expectLog(/Alice wins/i, 'Win announcement for Alice');
      this.expectNoLog('p1', 'No p1/p2 in logs - use player names');
      
      // Play winning sequence
      const moves = [
        { player: 'Alice', location: '/zones/board/0/0' },
        { player: 'Bob', location: '/zones/board/1/0' },
        { player: 'Alice', location: '/zones/board/0/1' },
        { player: 'Bob', location: '/zones/board/1/1' },
        { player: 'Alice', location: '/zones/board/0/2' } // Winning move
      ];
      
      for (const { player, location } of moves) {
        await this.executeAction(player, 'place', { location });
      }
      
      await this.wait(500);
      
      // Verify win
      this.assert.strictEqual(this.gameStatus.state, 'ended', 'Game should be ended');
      this.assert.strictEqual(this.gameStatus.winner, 'Alice', 'Alice should be winner');
      this.assert.strictEqual(this.gameStatus.tie, false, 'Should not be a tie');
      
      // Validate win was logged properly
      const logsValid = this.validateLogs();
      assert(logsValid, 'Win should be logged with player name');
    });
    
    // Add more win condition tests (vertical, diagonal, etc.)
  }

  // Category 4: Draw/Tie Conditions
  async testDrawConditions() {
    console.log('\n🤝 Testing Draw Conditions...');
    
    await this.testScenario('Game ends in draw when board is full', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Play a draw sequence (example for tic-tac-toe)
      const drawMoves = [
        { player: 'Alice', location: '/zones/board/0/0' },
        { player: 'Bob', location: '/zones/board/0/1' },
        { player: 'Alice', location: '/zones/board/0/2' },
        { player: 'Bob', location: '/zones/board/1/1' },
        { player: 'Alice', location: '/zones/board/1/0' },
        { player: 'Bob', location: '/zones/board/2/0' },
        { player: 'Alice', location: '/zones/board/1/2' },
        { player: 'Bob', location: '/zones/board/2/2' },
        { player: 'Alice', location: '/zones/board/2/1' }
      ];
      
      for (const { player, location } of drawMoves) {
        await this.executeAction(player, 'place', { location });
      }
      
      // Verify draw
      this.assert.strictEqual(this.gameStatus.state, 'ended', 'Game should be ended');
      this.assert.strictEqual(this.gameStatus.winner, null, 'Should have no winner');
      this.assert.strictEqual(this.gameStatus.tie, true, 'Should be a tie');
    });
  }

  // Category 5: Invalid Actions
  async testInvalidActions() {
    console.log('\n❌ Testing Invalid Actions...');
    
    await this.testScenario('Cannot move on opponent\'s turn', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Bob tries to move first (should fail)
      const error = await this.expectError(() =>
        this.executeAction('Bob', 'place', { location: '/zones/board/0/0' })
      );
      
      this.assert.ok(error, 'Should get error for wrong turn');
      this.assert.match(error.message, /not.*turn/i, 'Error should mention turn');
    });
    
    await this.testScenario('Cannot place on occupied space', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Alice places
      await this.executeAction('Alice', 'place', { location: '/zones/board/0/0' });
      
      // Bob tries same spot
      const error = await this.expectError(() =>
        this.executeAction('Bob', 'place', { location: '/zones/board/0/0' })
      );
      
      this.assert.ok(error, 'Should get error for occupied space');
    });
    
    await this.testScenario('Cannot make moves after game ends', async () => {
      // First, play to a win condition
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Play winning moves...
      // (abbreviated for example)
      
      // Try to make move after game end
      const error = await this.expectError(() =>
        this.executeAction('Bob', 'place', { location: '/zones/board/2/2' })
      );
      
      this.assert.ok(error, 'Should get error for post-game move');
    });
  }

  // Category 6: Edge Cases
  async testEdgeCases() {
    console.log('\n🔧 Testing Edge Cases...');
    
    await this.testScenario('Handles rapid consecutive moves', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Alice makes valid move
      await this.executeAction('Alice', 'place', { location: '/zones/board/0/0' });
      
      // Alice tries another move immediately (should fail)
      const error = await this.expectError(() =>
        this.executeAction('Alice', 'place', { location: '/zones/board/0/1' })
      );
      
      this.assert.ok(error, 'Should not allow consecutive moves from same player');
    });
    
    await this.testScenario('Handles disconnection and reconnection', async () => {
      const lobby = await this.createLobby();
      const alice = await this.connectPlayer(lobby.id, 'Alice');
      const bob = await this.connectPlayer(lobby.id, 'Bob');
      await this.startGame(lobby.id);
      
      // Alice disconnects
      alice.close();
      await this.wait(100);
      
      // Bob should still be able to see game state
      this.assert.ok(this.gameStarted, 'Game should remain started');
      
      // Alice reconnects
      const aliceReconnect = await this.connectPlayer(lobby.id, 'Alice');
      await this.wait(100);
      
      // Alice should receive current state
      // (verify through alice's connection)
    });
  }

  // Category 7: Concurrency Tests
  async testConcurrency() {
    console.log('\n⚡ Testing Concurrency...');
    
    await this.testScenario('Handles simultaneous actions correctly', async () => {
      const lobby = await this.createLobby();
      await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
      await this.startGame(lobby.id);
      
      // Both players try to claim different cells at the same time
      const alicePromise = this.executeAction('Alice', 'place', { 
        location: '/zones/board/0/0' 
      });
      
      const bobPromise = this.executeAction('Bob', 'place', { 
        location: '/zones/board/1/1' 
      });
      
      // Wait for both to complete
      const results = await Promise.allSettled([alicePromise, bobPromise]);
      
      // One should succeed, one should fail
      const succeeded = results.filter(r => r.status === 'fulfilled').length;
      const failed = results.filter(r => r.status === 'rejected').length;
      
      this.assert.strictEqual(succeeded, 1, 'Exactly one action should succeed');
      this.assert.strictEqual(failed, 1, 'Exactly one action should fail');
      
      // The successful one should be Alice's (her turn)
      if (results[0].status === 'fulfilled') {
        const cell = this.getState('/zones/board/0/0');
        this.assert.strictEqual(cell.entity, 'piece_Alice', 'Alice\'s move should succeed');
      }
    });
  }

  // Helper method for complex test setups
  async setupNearWinState() {
    const lobby = await this.createLobby();
    await this.connectPlayers(lobby.id, ['Alice', 'Bob']);
    await this.startGame(lobby.id);
    
    // Set up board state near win
    const setupMoves = [
      { player: 'Alice', location: '/zones/board/0/0' },
      { player: 'Bob', location: '/zones/board/1/0' },
      { player: 'Alice', location: '/zones/board/0/1' },
      { player: 'Bob', location: '/zones/board/2/0' }
    ];
    
    for (const { player, location } of setupMoves) {
      await this.executeAction(player, 'place', { location });
    }
    
    return lobby;
  }
}

// Run tests if executed directly
if (require.main === module) {
  const test = new ExampleGameTest();
  test.runAllTests().catch(console.error);
}

module.exports = ExampleGameTest;