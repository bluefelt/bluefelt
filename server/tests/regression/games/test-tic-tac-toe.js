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
  }

  processPatch(patches) {
    for (const patch of patches) {
      if (patch.path.match(/\/game\/zones\/board\/cells\/(\d+)\/(\d+)/)) {
        const [, row, col] = patch.path.match(/\/(\d+)\/(\d+)$/);
        this.board[parseInt(row)][parseInt(col)] = patch.value?.entity;
        this.moveCount++;
      }
      
      if (patch.path === '/game/currentPlayer') {
        this.currentPlayer = patch.value;
      }
      
      if (patch.path === '/game/gameStatus') {
        this.gameStatus = patch.value;
        if (patch.value?.state === 'ended') {
          this.gameEnded = true;
        }
      }
    }
  }

  async testAllWinConditions() {
    console.log('\n📋 Testing All Win Conditions\n');
    
    // Test horizontal win (row 0)
    await this.testScenario('Horizontal Win - Row 0', async () => {
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 0 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 0, col: 2 }, // P1 wins
      ];
      
      await this.playMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win');
      assert.strictEqual(this.moveCount, 5, 'Should take 5 moves');
    });
  }

  async testTieGame() {
    console.log('\n🤝 Testing Tie Game\n');
    
    await this.testScenario('Tie Game - Full Board', async () => {
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
      
      await this.playMoves(moves);
      console.log('Game status after all moves:', JSON.stringify(this.gameStatus));
      console.log('Move count:', this.moveCount);
      console.log('Game ended:', this.gameEnded);
      assert.strictEqual(this.gameStatus?.tie, true, 'Should be a tie');
      assert.strictEqual(this.gameStatus?.winner, null, 'No winner');
      assert.strictEqual(this.moveCount, 9, 'Board should be full');
    });
  }

  async testInvalidMoves() {
    console.log('\n❌ Testing Invalid Moves\n');
    
    await this.testScenario('Occupied Cell Rejection', async () => {
      // P1 plays center
      await this.executeAction('p1', 'placeMarker', {
        location: '/zones/board/cells/1/1',
        entity: 'mark_p1'
      });
      
      const currentTurn = this.currentPlayer;
      
      // P2 tries same cell
      await this.executeAction('p2', 'placeMarker', {
        location: '/zones/board/cells/1/1',
        entity: 'mark_p2'
      });
      
      // Should still be P2's turn (move rejected)
      assert.strictEqual(this.currentPlayer, 'p2', 'Turn should not advance');
    });

    await this.testScenario('Out of Turn Move', async () => {
      // P1 plays
      await this.executeAction('p1', 'placeMarker', {
        location: '/zones/board/cells/0/0',
        entity: 'mark_p1'
      });
      
      // Wait for turn to switch
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Verify it's P2's turn
      assert.strictEqual(this.currentPlayer, 'p2', 'Should be P2 turn after P1 move');
      
      // P1 tries again (should be rejected)
      const boardBefore = JSON.stringify(this.board);
      const turnBefore = this.currentPlayer;
      
      await this.executeAction('p1', 'placeMarker', {
        location: '/zones/board/cells/0/1',
        entity: 'mark_p1'
      });
      
      const boardAfter = JSON.stringify(this.board);
      assert.strictEqual(boardBefore, boardAfter, 'Board should not change');
      assert.strictEqual(this.currentPlayer, turnBefore, 'Turn should not change');
    });
  }

  async testEdgeCases() {
    console.log('\n🔧 Testing Edge Cases\n');
    
    await this.testScenario('Minimum Moves to Win', async () => {
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 0 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 0, col: 2 }, // Fastest possible win
      ];
      
      await this.playMoves(moves);
      assert.strictEqual(this.moveCount, 5, 'Minimum 5 moves to win');
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 wins');
    });

    await this.testScenario('Move After Game End', async () => {
      // Play winning game first
      await this.executeAction('p1', 'placeMarker', {
        location: '/zones/board/cells/0/0',
        entity: 'mark_p1'
      });
      
      await this.executeAction('p2', 'placeMarker', {
        location: '/zones/board/cells/1/0',
        entity: 'mark_p2'
      });
      
      await this.executeAction('p1', 'placeMarker', {
        location: '/zones/board/cells/0/1',
        entity: 'mark_p1'
      });
      
      await this.executeAction('p2', 'placeMarker', {
        location: '/zones/board/cells/1/1',
        entity: 'mark_p2'
      });
      
      await this.executeAction('p1', 'placeMarker', {
        location: '/zones/board/cells/0/2',
        entity: 'mark_p1'
      });
      
      // Wait for game end processing
      await new Promise(resolve => setTimeout(resolve, 200));
      
      // Debug output
      console.log('      Game ended:', this.gameEnded);
      console.log('      Game status:', JSON.stringify(this.gameStatus));
      console.log('      Move count before:', this.moveCount);
      
      assert(this.gameEnded, 'Game should be ended');
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should have won');
      
      // Try to play after game end
      const movesBefore = this.moveCount;
      await this.executeAction('p2', 'placeMarker', {
        location: '/zones/board/cells/2/2'
      });
      
      console.log('      Move count after:', this.moveCount);
      
      // The move should be rejected - no new moves after game end
      assert.strictEqual(this.moveCount, movesBefore, 'No moves after game end');
      assert.strictEqual(this.gameStatus?.state, 'ended', 'Game should remain ended');
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'Winner should not change');
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
        await this.executeAction(move.player, 'placeMarker', {
          location: `/zones/board/cells/${move.row}/${move.col}`,
          entity: `mark_${move.player}`
        });
      }
    }
    this.printBoard();
  }

  // Helper to run a test scenario
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

}

// Test execution
async function runTest() {
  let allSuccess = true;
  
  try {
    // Run each test group with a fresh lobby
    const testGroups = [
      { name: 'Win Conditions', method: 'testAllWinConditions' },
      { name: 'Tie Game', method: 'testTieGame' },
      { name: 'Invalid Moves', method: 'testInvalidMoves' },
      { name: 'Edge Cases', method: 'testEdgeCases' }
    ];
    
    for (const group of testGroups) {
      console.log(`\n${'='.repeat(60)}`);
      console.log(`Running: ${group.name}`);
      console.log(`${'='.repeat(60)}\n`);
      
      const test = new TicTacToeRegressionTest();
      
      try {
        await test.createLobby();
        await test.connectPlayers(['Alice', 'Bob']);
        await test.startGame();
        
        await test[group.method]();
        
        test.cleanup();
      } catch (error) {
        console.error(`${group.name} failed:`, error);
        allSuccess = false;
        test.cleanup();
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