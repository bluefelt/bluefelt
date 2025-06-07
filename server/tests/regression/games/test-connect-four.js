const { GameTestFramework } = require('../framework/GameTestFramework.js');
const assert = require('assert');

/**
 * Complete regression test suite for Connect Four
 * Tests all possible game outcomes, gravity mechanics, and edge cases
 */
class ConnectFourRegressionTest extends GameTestFramework {
  constructor() {
    super('connect-four');
    this.board = Array(6).fill(null).map(() => Array(7).fill(null)); // 6 rows, 7 columns
    this.moveCount = 0;
    this.columnHeights = Array(7).fill(0); // Track height of each column
  }

  processPatch(patches) {
    for (const patch of patches) {
      // Board updates with gravity
      if (patch.path.match(/\/game\/zones\/board\/cells\/(\d+)\/(\d+)/)) {
        const [, row, col] = patch.path.match(/\/(\d+)\/(\d+)$/);
        const rowIdx = parseInt(row);
        const colIdx = parseInt(col);
        this.board[rowIdx][colIdx] = patch.value?.entity;
        
        // Update column height
        if (patch.value?.entity) {
          this.columnHeights[colIdx] = Math.max(this.columnHeights[colIdx], 6 - rowIdx);
          this.moveCount++;
        }
      }
      
      if (patch.path === '/game/currentPlayer') {
        this.currentPlayer = patch.value;
      }
      
      if (patch.path === '/game/gameStatus') {
        this.gameStatus = patch.value;
      }
    }
  }

  async testAllWinConditions() {
    console.log('\n📋 Testing All Win Conditions\n');
    
    // Test horizontal win
    await this.testScenario('Horizontal Win - Bottom Row', async () => {
      const moves = [
        { player: 'p1', col: 0 },
        { player: 'p2', col: 0 },
        { player: 'p1', col: 1 },
        { player: 'p2', col: 1 },
        { player: 'p1', col: 2 },
        { player: 'p2', col: 2 },
        { player: 'p1', col: 3 }, // P1 wins horizontally
      ];
      
      await this.playMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win horizontally');
      assert.strictEqual(this.moveCount, 7, 'Should take 7 moves');
    });

    await this.testScenario('Vertical Win', async () => {
      const moves = [
        { player: 'p1', col: 0 },
        { player: 'p2', col: 1 },
        { player: 'p1', col: 0 },
        { player: 'p2', col: 1 },
        { player: 'p1', col: 0 },
        { player: 'p2', col: 1 },
        { player: 'p1', col: 0 }, // P1 wins vertically in column 0
      ];
      
      await this.playMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win vertically');
    });

    await this.testScenario('Diagonal Win - Up Right', async () => {
      // Build a diagonal from bottom-left to top-right
      // Strategy: P1 gets (5,0), (4,1), (3,2), (2,3) for diagonal win
      const moves = [
        { player: 'p1', col: 0 }, // (5,0) - P1's first diagonal piece
        { player: 'p2', col: 1 }, // (5,1) - P2 blocks but doesn't interfere
        { player: 'p1', col: 1 }, // (4,1) - P1's second diagonal piece
        { player: 'p2', col: 6 }, // (5,6) - P2 plays elsewhere
        { player: 'p1', col: 2 }, // (5,2) - P1 bottom of column 2
        { player: 'p2', col: 6 }, // (4,6) - P2 plays elsewhere
        { player: 'p1', col: 2 }, // (4,2) - P1 builds up column 2
        { player: 'p2', col: 5 }, // (5,5) - P2 plays elsewhere
        { player: 'p1', col: 2 }, // (3,2) - P1's third diagonal piece
        { player: 'p2', col: 5 }, // (4,5) - P2 plays elsewhere
        { player: 'p1', col: 3 }, // (5,3) - P1 bottom of column 3
        { player: 'p2', col: 5 }, // (3,5) - P2 builds column 5
        { player: 'p1', col: 3 }, // (4,3) - P1 builds column 3
        { player: 'p2', col: 4 }, // (5,4) - P2 plays column 4
        { player: 'p1', col: 3 }, // (3,3) - P1 builds column 3
        { player: 'p2', col: 4 }, // (4,4) - P2 builds column 4
        { player: 'p1', col: 3 }, // (2,3) - P1's fourth diagonal piece - WINS!
      ];
      
      await this.playMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win diagonally');
    });

    await this.testScenario('Diagonal Win - Up Left', async () => {
      // Build a diagonal from bottom-right to top-left
      // Strategy: P1 gets (5,6), (4,5), (3,4), (2,3) for diagonal win
      const moves = [
        { player: 'p1', col: 6 }, // (5,6) - P1's first diagonal piece
        { player: 'p2', col: 0 }, // (5,0) - P2 plays elsewhere
        { player: 'p1', col: 5 }, // (5,5) - P1 bottom of column 5
        { player: 'p2', col: 0 }, // (4,0) - P2 plays elsewhere
        { player: 'p1', col: 5 }, // (4,5) - P1's second diagonal piece
        { player: 'p2', col: 1 }, // (5,1) - P2 plays elsewhere
        { player: 'p1', col: 4 }, // (5,4) - P1 bottom of column 4
        { player: 'p2', col: 1 }, // (4,1) - P2 plays elsewhere
        { player: 'p1', col: 4 }, // (4,4) - P1 builds column 4
        { player: 'p2', col: 2 }, // (5,2) - P2 plays elsewhere
        { player: 'p1', col: 4 }, // (3,4) - P1's third diagonal piece
        { player: 'p2', col: 2 }, // (4,2) - P2 plays elsewhere
        { player: 'p1', col: 3 }, // (5,3) - P1 bottom of column 3
        { player: 'p2', col: 2 }, // (3,2) - P2 plays elsewhere
        { player: 'p1', col: 3 }, // (4,3) - P1 builds column 3
        { player: 'p2', col: 2 }, // (2,2) - P2 plays elsewhere
        { player: 'p1', col: 3 }, // (3,3) - P1 builds column 3
        { player: 'p2', col: 1 }, // (3,1) - P2 plays elsewhere
        { player: 'p1', col: 3 }, // (2,3) - P1's fourth diagonal piece - WINS!
      ];
      
      await this.playMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win diagonally');
    });
  }

  async testGravityMechanics() {
    console.log('\n🌍 Testing Gravity Mechanics\n');
    
    await this.testScenario('Discs Fall to Bottom', async () => {
      // Drop disc in empty column
      await this.executeAction('p1', 'dropDisc', {
        targetColumn: 3
      });
      
      // Verify disc is at bottom (row 5)
      assert.strictEqual(this.board[5][3], 'disc_p1', 'Disc should fall to bottom');
      assert.strictEqual(this.columnHeights[3], 1, 'Column height should be 1');
    });

    await this.testScenario('Discs Stack Properly', async () => {
      // Stack discs in column 0
      await this.executeAction('p1', 'dropDisc', { targetColumn: 0 });
      await this.executeAction('p2', 'dropDisc', { targetColumn: 0 });
      await this.executeAction('p1', 'dropDisc', { targetColumn: 0 });
      
      // Verify stacking from bottom to top
      assert.strictEqual(this.board[5][0], 'disc_p1', 'Bottom disc should be P1');
      assert.strictEqual(this.board[4][0], 'disc_p2', 'Middle disc should be P2');
      assert.strictEqual(this.board[3][0], 'disc_p1', 'Top disc should be P1');
      assert.strictEqual(this.columnHeights[0], 3, 'Column height should be 3');
    });
  }

  async testTieGame() {
    console.log('\n🤝 Testing Tie Game\n');
    
    await this.testScenario('Board Full Without Winner', async () => {
      // Fill board in pattern that prevents any 4-in-a-row
      // Column pattern: p1,p2,p1,p2,p1,p2 (prevents vertical wins)
      // Alternating pattern prevents horizontal wins
      const moves = [];
      for (let col = 0; col < 7; col++) {
        for (let row = 0; row < 6; row++) {
          const player = (col % 2 === row % 2) ? 'p1' : 'p2';
          moves.push({ player: moves.length % 2 === 0 ? 'p1' : 'p2', col });
        }
      }
      
      await this.playMoves(moves.slice(0, 42)); // Play all 42 moves
      
      assert.strictEqual(this.moveCount, 42, 'Board should be full');
      assert.strictEqual(this.gameStatus?.tie, true, 'Should be a tie');
      assert.strictEqual(this.gameStatus?.winner, null, 'No winner');
    });
  }

  async testInvalidMoves() {
    console.log('\n❌ Testing Invalid Moves\n');
    
    await this.testScenario('Full Column Rejection', async () => {
      // Fill column 2 completely
      for (let i = 0; i < 6; i++) {
        const player = i % 2 === 0 ? 'p1' : 'p2';
        await this.executeAction(player, 'dropDisc', { targetColumn: 2 });
      }
      
      assert.strictEqual(this.columnHeights[2], 6, 'Column should be full');
      
      // Try to add one more disc
      const movesBefore = this.moveCount;
      await this.executeAction(this.currentPlayer, 'dropDisc', { targetColumn: 2 });
      
      assert.strictEqual(this.moveCount, movesBefore, 'Move should be rejected');
    });

    await this.testScenario('Out of Turn Move', async () => {
      // P1 plays
      await this.executeAction('p1', 'dropDisc', { targetColumn: 0 });
      
      // P1 tries again (should be P2's turn)
      const boardBefore = JSON.stringify(this.board);
      await this.executeAction('p1', 'dropDisc', { targetColumn: 1 });
      
      const boardAfter = JSON.stringify(this.board);
      assert.strictEqual(boardBefore, boardAfter, 'Board should not change');
      assert.strictEqual(this.currentPlayer, 'p2', 'Should be P2 turn');
    });

    await this.testScenario('Invalid Column Numbers', async () => {
      const movesBefore = this.moveCount;
      
      // Try negative column
      await this.executeAction('p1', 'dropDisc', { targetColumn: -1 });
      assert.strictEqual(this.moveCount, movesBefore, 'Negative column rejected');
      
      // Try column beyond board
      await this.executeAction('p1', 'dropDisc', { targetColumn: 7 });
      assert.strictEqual(this.moveCount, movesBefore, 'Out of bounds column rejected');
    });
  }

  async testEdgeCases() {
    console.log('\n🔧 Testing Edge Cases\n');
    
    await this.testScenario('Minimum Moves to Win', async () => {
      // P1 can win in 7 moves (4 discs in bottom row)
      const moves = [
        { player: 'p1', col: 0 },
        { player: 'p2', col: 0 },
        { player: 'p1', col: 1 },
        { player: 'p2', col: 1 },
        { player: 'p1', col: 2 },
        { player: 'p2', col: 2 },
        { player: 'p1', col: 3 }, // Fastest possible win
      ];
      
      await this.playMoves(moves);
      assert.strictEqual(this.moveCount, 7, 'Minimum 7 moves to win');
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 wins');
    });

    await this.testScenario('Win at Board Edge', async () => {
      // Test winning at right edge
      const moves = [
        { player: 'p1', col: 6 },
        { player: 'p2', col: 0 },
        { player: 'p1', col: 5 },
        { player: 'p2', col: 0 },
        { player: 'p1', col: 4 },
        { player: 'p2', col: 0 },
        { player: 'p1', col: 3 }, // Win at right edge
      ];
      
      await this.playMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'Should win at edge');
    });

    await this.testScenario('Win at Top Row', async () => {
      // Fill columns to test win at top
      const moves = [];
      
      // Fill columns 0-3 to second-to-top row
      for (let row = 0; row < 5; row++) {
        for (let col = 0; col < 4; col++) {
          const player = (row + col) % 2 === 0 ? 'p2' : 'p1';
          moves.push({ player: moves.length % 2 === 0 ? 'p1' : 'p2', col });
        }
      }
      
      // Now P1 can win at top row
      moves.push({ player: 'p1', col: 0 });
      moves.push({ player: 'p2', col: 4 });
      moves.push({ player: 'p1', col: 1 });
      moves.push({ player: 'p2', col: 4 });
      moves.push({ player: 'p1', col: 2 });
      moves.push({ player: 'p2', col: 4 });
      moves.push({ player: 'p1', col: 3 }); // Win at top row
      
      await this.playMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'Should win at top row');
    });

    await this.testScenario('Move After Game End', async () => {
      // Play winning game
      const winMoves = [
        { player: 'p1', col: 0 },
        { player: 'p2', col: 1 },
        { player: 'p1', col: 0 },
        { player: 'p2', col: 1 },
        { player: 'p1', col: 0 },
        { player: 'p2', col: 1 },
        { player: 'p1', col: 0 }, // P1 wins
      ];
      
      await this.playMoves(winMoves);
      assert(this.gameEnded, 'Game should be ended');
      
      // Try to play after game end
      const movesBefore = this.moveCount;
      await this.executeAction('p2', 'dropDisc', { targetColumn: 6 });
      
      assert.strictEqual(this.moveCount, movesBefore, 'No moves after game end');
    });
  }

  // Helper to play a sequence of moves
  async playMoves(moves) {
    this.board = Array(6).fill(null).map(() => Array(7).fill(null));
    this.columnHeights = Array(7).fill(0);
    this.moveCount = 0;
    this.gameEnded = false;
    this.gameStatus = null;
    
    for (const move of moves) {
      if (this.gameEnded) break;
      if (this.currentPlayer === move.player) {
        await this.executeAction(move.player, 'dropDisc', {
          targetColumn: move.col
        });
      }
    }
  }

  // Helper to run a test scenario
  async testScenario(name, testFunc) {
    console.log(`  Testing: ${name}`);
    
    // Reset game state for each scenario
    await this.resetGame();
    
    try {
      await testFunc();
      console.log(`    ✅ PASSED`);
    } catch (error) {
      console.log(`    ❌ FAILED: ${error.message}`);
      this.printBoard();
      throw error;
    }
  }

  async resetGame() {
    // Close existing connections
    this.cleanup();
    this.players.clear();
    
    // Reset state
    this.board = Array(6).fill(null).map(() => Array(7).fill(null));
    this.columnHeights = Array(7).fill(0);
    this.moveCount = 0;
    this.gameEnded = false;
    this.gameStatus = null;
    this.currentPlayer = null;
    
    // Create new game
    await this.createLobby();
    await this.connectPlayers(['Alice', 'Bob']);
    await this.startGame();
  }

  printBoard() {
    console.log('\nBoard (top to bottom):');
    for (let row = 0; row < 6; row++) {
      const rowStr = this.board[row].map(cell => {
        if (!cell) return '·';
        return cell.includes('p1') ? '🔴' : '🟡';
      }).join(' ');
      console.log(`  Row ${row}: ${rowStr}`);
    }
    console.log('  Col:   0 1 2 3 4 5 6');
  }

  // Main test runner
  async runAllTests() {
    console.log('\n🎮 CONNECT FOUR REGRESSION TEST SUITE\n');
    
    try {
      await this.testGravityMechanics();
      await this.testAllWinConditions();
      await this.testTieGame();
      await this.testInvalidMoves();
      await this.testEdgeCases();
      
      console.log('\n✅ All tests passed!\n');
      return true;
    } catch (error) {
      console.error('\n❌ Test suite failed:', error);
      return false;
    }
  }
}

// Test execution
async function runTest() {
  const test = new ConnectFourRegressionTest();
  
  try {
    await test.createLobby();
    await test.connectPlayers(['Alice', 'Bob']);
    await test.startGame();
    
    const success = await test.runAllTests();
    
    test.cleanup();
    process.exit(success ? 0 : 1);
    
  } catch (error) {
    console.error('Test setup failed:', error);
    test.cleanup();
    process.exit(1);
  }
}

module.exports = { ConnectFourRegressionTest };

if (require.main === module) {
  runTest();
}