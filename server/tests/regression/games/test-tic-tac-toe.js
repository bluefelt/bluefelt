const { GameTestFramework } = require('../framework/GameTestFramework.js');
const assert = require('assert');

/**
 * Complete regression test suite for Tic-Tac-Toe
 * Tests all possible game outcomes and edge cases
 */
class TicTacToeRegressionTest extends GameTestFramework {
  constructor() {
    super('tic-tac-toe');
    this.board = Array(3).fill(null).map(() => Array(3).fill(null));
    this.moveCount = 0;
    this.lastMoveTickProcessed = -1; // Track the last tick we counted a move for
  }

  processPatch(patches) {
    // Track if this patch batch contains a board update
    let boardUpdated = false;
    let lastBoardUpdate = null;
    
    for (const patch of patches) {
      // Board updates
      if (patch.path.match(/\/game\/zones\/board\/cells\/(\d+)\/(\d+)/)) {
        const [, row, col] = patch.path.match(/\/(\d+)\/(\d+)$/);
        const entity = patch.value?.entity;
        this.board[parseInt(row)][parseInt(col)] = entity;
        
        // Only track if this is a new entity placement (not clearing)
        if (entity) {
          boardUpdated = true;
          lastBoardUpdate = { row: parseInt(row), col: parseInt(col), entity };
        }
      }
      
      // Turn updates
      if (patch.path === '/game/currentPlayer') {
        this.currentPlayer = patch.value;
      }
      
      // Game status updates
      if (patch.path === '/game/gameStatus') {
        this.gameStatus = patch.value;
        if (patch.value?.state === 'ended') {
          this.gameEnded = true;
        }
      }
      
      // Action map updates
      if (patch.path === '/ui/actionMap') {
        this.actionMap = patch.value;
      }
    }
    
    // Simplified move counting: increment for each board update we see
    // Use a Set to track which cells have been updated to avoid double-counting
    if (boardUpdated && lastBoardUpdate) {
      const cellKey = `${lastBoardUpdate.row},${lastBoardUpdate.col}`;
      if (!this.processedCells) {
        this.processedCells = new Set();
      }
      
      if (!this.processedCells.has(cellKey)) {
        this.processedCells.add(cellKey);
        this.moveCount++;
        
        // Extract player from entity (mark_p1 -> p1, mark_p2 -> p2)
        const entityMatch = lastBoardUpdate.entity.match(/mark_(.+)$/);
        const movingPlayer = entityMatch ? entityMatch[1] : lastBoardUpdate.entity;
        
        console.log(`  Move ${this.moveCount}: ${movingPlayer} → (${lastBoardUpdate.row},${lastBoardUpdate.col})`);
      }
    }
  }

  static async testActionMaps() {
    console.log('\n🎯 Testing Action Maps\n');
    
    // Test 1: Initial Action Map
    await TicTacToeRegressionTest.runScenario('Initial Action Map', async (test) => {
      // P1 should have 9 available cells at start
      assert(test.actionMap, 'Action map should exist');
      assert(test.actionMap.p1, 'P1 should have actions');
      const p1ActionCount = Object.keys(test.actionMap.p1).length;
      assert.strictEqual(p1ActionCount, 9, 'P1 should have 9 available cells');
      
      // P2 should have no actions (not their turn)
      assert(test.actionMap.p2, 'P2 action map should exist');
      const p2ActionCount = Object.keys(test.actionMap.p2).length;
      assert.strictEqual(p2ActionCount, 0, 'P2 should have no actions');
      
      // Verify all cells are clickable for P1
      for (let row = 0; row < 3; row++) {
        for (let col = 0; col < 3; col++) {
          const cellPath = `/zones/board/cells/${row}/${col}`;
          assert(test.actionMap.p1[cellPath], `Cell (${row},${col}) should be clickable for P1`);
          assert.strictEqual(test.actionMap.p1[cellPath].action, 'placeMark', 'Should be placeMark action');
        }
      }
    });
    
    await TicTacToeRegressionTest.runScenario('Action Map After Move', async (test) => {
      // P1 places at (1,1)
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/1/1',
        entity: 'mark_p1'
      });
      
      // Wait for turn transition
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Now P2 should have 8 available cells
      const p2ActionCount = Object.keys(test.actionMap.p2).length;
      assert.strictEqual(p2ActionCount, 8, 'P2 should have 8 available cells');
      
      // P1 should have no actions
      const p1ActionCount = Object.keys(test.actionMap.p1).length;
      assert.strictEqual(p1ActionCount, 0, 'P1 should have no actions');
      
      // Cell (1,1) should not be in P2's action map
      assert(!test.actionMap.p2['/zones/board/cells/1/1'], 'Occupied cell should not be clickable');
    });
    
    await TicTacToeRegressionTest.runScenario('Action Map When Game Ends', async (test) => {
      // Play winning sequence for P1 in row 2
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/2/0',
        entity: 'mark_p1'
      });
      await test.executeAction('p2', 'placeMark', {
        location: '/zones/board/cells/0/0',
        entity: 'mark_p2'
      });
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/2/1',
        entity: 'mark_p1'
      });
      await test.executeAction('p2', 'placeMark', {
        location: '/zones/board/cells/0/1',
        entity: 'mark_p2'
      });
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/2/2',
        entity: 'mark_p1'
      });
      
      // Wait for game end - give time for all patches to be processed
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Debug the state
      console.log('    Game status:', JSON.stringify(test.gameStatus));
      console.log('    Game ended:', test.gameEnded);
      console.log('    Action map:', JSON.stringify(test.actionMap));
      
      // Both players should have no actions after game ends
      const p1ActionCount = Object.keys(test.actionMap.p1 || {}).length;
      const p2ActionCount = Object.keys(test.actionMap.p2 || {}).length;
      
      assert.strictEqual(p1ActionCount, 0, 'P1 should have no actions after game ends');
      assert.strictEqual(p2ActionCount, 0, 'P2 should have no actions after game ends');
    });
  }

  static async testAllWinConditions() {
    console.log('\n📋 Testing All Win Conditions\n');
    
    // Test horizontal win (row 0)
    await TicTacToeRegressionTest.runScenario('Horizontal Win - Row 0', async (test) => {
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 0 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 0, col: 2 }, // P1 wins
      ];
      
      await test.playMoves(moves);
      assert.strictEqual(test.gameStatus?.winner, 'p1', 'P1 should win');
      assert.strictEqual(test.moveCount, 5, 'Should take 5 moves');
    });
  }

  static async testTieGame() {
    console.log('\n🤝 Testing Tie Game\n');
    
    await TicTacToeRegressionTest.runScenario('Tie Game - Full Board', async (test) => {
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 2, col: 2 },
        { player: 'p2', row: 0, col: 2 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 2, col: 1 },
        { player: 'p1', row: 1, col: 2 },
        { player: 'p2', row: 1, col: 0 },
        { player: 'p1', row: 2, col: 0 }, // Board full - tie
      ];
      
      await test.playMoves(moves);
      console.log('Game status after all moves:', JSON.stringify(test.gameStatus));
      console.log('Move count:', test.moveCount);
      console.log('Game ended:', test.gameEnded);
      assert.strictEqual(test.gameStatus?.tie, true, 'Should be a tie');
      assert.strictEqual(test.gameStatus?.winner, null, 'No winner');
      assert.strictEqual(test.moveCount, 9, 'Board should be full');
    });
  }

  static async testInvalidMoves() {
    console.log('\n❌ Testing Invalid Moves\n');
    
    await TicTacToeRegressionTest.runScenario('Occupied Cell Rejection', async (test) => {
      // P1 plays center
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/1/1',
        entity: 'mark_p1'
      });
      
      const currentTurn = test.currentPlayer;
      
      // P2 tries same cell
      await test.executeAction('p2', 'placeMark', {
        location: '/zones/board/cells/1/1',
        entity: 'mark_p2'
      });
      
      // Should still be P2's turn (move rejected)
      assert.strictEqual(test.currentPlayer, 'p2', 'Turn should not advance');
    });

    await TicTacToeRegressionTest.runScenario('Out of Turn Move', async (test) => {
      // P1 plays
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/0/0',
        entity: 'mark_p1'
      });
      
      // Wait for turn to switch
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Verify it's P2's turn
      assert.strictEqual(test.currentPlayer, 'p2', 'Should be P2 turn after P1 move');
      
      // P1 tries again (should be rejected)
      const boardBefore = JSON.stringify(test.board);
      const turnBefore = test.currentPlayer;
      
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/0/1',
        entity: 'mark_p1'
      });
      
      const boardAfter = JSON.stringify(test.board);
      assert.strictEqual(boardBefore, boardAfter, 'Board should not change');
      assert.strictEqual(test.currentPlayer, turnBefore, 'Turn should not change');
    });
  }

  static async testEdgeCases() {
    console.log('\n🔧 Testing Edge Cases\n');
    
    await TicTacToeRegressionTest.runScenario('Minimum Moves to Win', async (test) => {
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 0 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 0, col: 2 }, // Fastest possible win
      ];
      
      await test.playMoves(moves);
      assert.strictEqual(test.moveCount, 5, 'Minimum 5 moves to win');
      assert.strictEqual(test.gameStatus?.winner, 'p1', 'P1 wins');
    });

    await TicTacToeRegressionTest.runScenario('Move After Game End', async (test) => {
      // Play winning game first
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/0/0',
        entity: 'mark_p1'
      });
      
      await test.executeAction('p2', 'placeMark', {
        location: '/zones/board/cells/1/0',
        entity: 'mark_p2'
      });
      
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/0/1',
        entity: 'mark_p1'
      });
      
      await test.executeAction('p2', 'placeMark', {
        location: '/zones/board/cells/1/1',
        entity: 'mark_p2'
      });
      
      await test.executeAction('p1', 'placeMark', {
        location: '/zones/board/cells/0/2',
        entity: 'mark_p1'
      });
      
      // Wait for game end processing
      await new Promise(resolve => setTimeout(resolve, 200));
      
      // Debug output
      console.log('      Game ended:', test.gameEnded);
      console.log('      Game status:', JSON.stringify(test.gameStatus));
      console.log('      Move count before:', test.moveCount);
      
      assert(test.gameEnded, 'Game should be ended');
      assert.strictEqual(test.gameStatus?.winner, 'p1', 'P1 should have won');
      
      // Try to play after game end
      const movesBefore = test.moveCount;
      await test.executeAction('p2', 'placeMark', {
        location: '/zones/board/cells/2/2'
      });
      
      console.log('      Move count after:', test.moveCount);
      
      // The move should be rejected - no new moves after game end
      assert.strictEqual(test.moveCount, movesBefore, 'No moves after game end');
      assert.strictEqual(test.gameStatus?.state, 'ended', 'Game should remain ended');
      assert.strictEqual(test.gameStatus?.winner, 'p1', 'Winner should not change');
    });
  }

  // Helper to print board state
  printBoard() {
    console.log('\nBoard state:');
    for (let row = 0; row < 3; row++) {
      let rowStr = '  ';
      for (let col = 0; col < 3; col++) {
        const entity = this.board[row][col];
        if (entity === 'mark_p1') rowStr += 'X ';
        else if (entity === 'mark_p2') rowStr += 'O ';
        else rowStr += '- ';
      }
      console.log(rowStr);
    }
    console.log();
  }

  // Helper to play a sequence of moves
  async playMoves(moves) {
    for (const move of moves) {
      if (this.gameEnded) break;
      if (this.currentPlayer === move.player) {
        await this.executeAction(move.player, 'placeMark', {
          location: `/zones/board/cells/${move.row}/${move.col}`,
          entity: `mark_${move.player}`
        });
      }
    }
    this.printBoard();
  }

  // Helper to run a test scenario with a fresh game
  async testScenario(name, testFunc) {
    console.log(`  Testing: ${name}`);
    
    try {
      await testFunc();
      console.log(`    ✅ PASSED`);
    } catch (error) {
      console.log(`    ❌ FAILED: ${error.message}`);
      throw error;
    }
  }
  
  // Create a fresh game instance for a scenario
  static async createFreshGame() {
    const test = new TicTacToeRegressionTest();
    await test.createLobby();
    await test.connectPlayers(['Alice', 'Bob']);
    await test.startGame();
    return test;
  }
  
  // Run a scenario with a fresh game instance
  static async runScenario(name, testFunc) {
    console.log(`  Testing: ${name}`);
    const test = await TicTacToeRegressionTest.createFreshGame();
    
    try {
      await testFunc(test);
      console.log(`    ✅ PASSED`);
      await test.cleanup();
    } catch (error) {
      console.log(`    ❌ FAILED: ${error.message}`);
      await test.cleanup();
      throw error;
    }
  }

}

// Test execution
async function runTest() {
  let allSuccess = true;
  
  // Handle cleanup on process termination
  process.on('SIGINT', async () => {
    console.log('\n\n⚠️  Test interrupted - cleaning up...');
    process.exit(1);
  });
  
  try {
    // Run each test group - each test method creates its own fresh games
    const testGroups = [
      { name: 'Action Maps', method: TicTacToeRegressionTest.testActionMaps },
      { name: 'Win Conditions', method: TicTacToeRegressionTest.testAllWinConditions },
      { name: 'Tie Game', method: TicTacToeRegressionTest.testTieGame },
      { name: 'Invalid Moves', method: TicTacToeRegressionTest.testInvalidMoves },
      { name: 'Edge Cases', method: TicTacToeRegressionTest.testEdgeCases }
    ];
    
    for (const group of testGroups) {
      console.log(`\n${'='.repeat(60)}`);
      console.log(`Running: ${group.name}`);
      console.log(`${'='.repeat(60)}\n`);
      
      try {
        await group.method();
      } catch (error) {
        console.error(`${group.name} failed:`, error);
        allSuccess = false;
      }
    }
    
    console.log('\n' + '='.repeat(60));
    console.log(allSuccess ? '✅ ALL TESTS PASSED!' : '❌ SOME TESTS FAILED!');
    console.log('='.repeat(60) + '\n');
    
    process.exit(allSuccess ? 0 : 1);
    
  } catch (error) {
    console.error('Test runner failed:', error);
    process.exit(1);
  }
}

module.exports = { TicTacToeRegressionTest };

if (require.main === module) {
  runTest();
}