const { GameTestFramework } = require('../framework/GameTestFramework.js');
const assert = require('assert');

/**
 * Complete regression test suite for Three Men's Morris
 * Tests placement phase, movement phase, win conditions, and phase transitions
 */
class ThreeMensMorrisRegressionTest extends GameTestFramework {
  constructor() {
    super('three-mens-morris');
    this.board = Array(3).fill(null).map(() => Array(3).fill(null));
    this.pieceCounts = { p1: 0, p2: 0 };
    this.currentPhase = 'placement';
    this.selectedPiece = null;
  }

  processPatch(patches) {
    for (const patch of patches) {
      // Board updates
      if (patch.path.match(/\/game\/zones\/board\/cells\/(\d+)\/(\d+)/)) {
        const [, row, col] = patch.path.match(/\/(\d+)\/(\d+)$/);
        const rowIdx = parseInt(row);
        const colIdx = parseInt(col);
        const entity = patch.value?.entity || null;
        
        // Track piece placement/movement
        if (!this.board[rowIdx][colIdx] && entity) {
          // Piece added
          const player = entity.includes('p1') ? 'p1' : 'p2';
          if (this.currentPhase === 'placement') {
            this.pieceCounts[player]++;
          }
        }
        
        this.board[rowIdx][colIdx] = entity;
      }
      
      // Phase transitions
      if (patch.path === '/game/phases/game') {
        this.currentPhase = patch.value;
        console.log(`  Phase changed to: ${this.currentPhase}`);
      }
      
      // Selection tracking
      if (patch.path.startsWith('/game/selection/')) {
        this.selectedPiece = patch.op === 'remove' ? null : patch.value;
      }
      
      if (patch.path === '/game/currentPlayer') {
        this.currentPlayer = patch.value;
      }
      
      if (patch.path === '/game/gameStatus') {
        this.gameStatus = patch.value;
        if (this.gameStatus && this.gameStatus.state === 'ended') {
          this.gameEnded = true;
        }
      }
    }
  }

  async testPlacementPhase() {
    console.log('\n📋 Testing Placement Phase\n');
    
    await this.testScenario('Placement Flow to Movement Phase', async () => {
      // Players place pieces without any winning lines - game should transition to movement
      const moves = [
        { player: 'p1', action: 'placeToken', row: 0, col: 0 },
        { player: 'p2', action: 'placeToken', row: 1, col: 1 },
        { player: 'p1', action: 'placeToken', row: 0, col: 1 },
        { player: 'p2', action: 'placeToken', row: 2, col: 0 },
        { player: 'p1', action: 'placeToken', row: 1, col: 2 },
        { player: 'p2', action: 'placeToken', row: 2, col: 2 },
      ];
      
      for (const move of moves) {
        await this.executeAction(move.player, move.action, {
          target: `/zones/board/cells/${move.row}/${move.col}`,
          entity: `piece_${move.player}`
        });
      }
      
      assert.strictEqual(this.pieceCounts.p1, 3, 'P1 should have 3 pieces');
      assert.strictEqual(this.pieceCounts.p2, 3, 'P2 should have 3 pieces');
      assert.strictEqual(this.currentPhase, 'movement', 'Should transition to movement phase');
    });

    await this.testScenario('Cannot Place More Than 3 Pieces', async () => {
      // First place 3 pieces for P1
      await this.executeAction('p1', 'placeToken', {
        target: '/zones/board/cells/0/0',
        entity: 'piece_p1'
      });
      await this.executeAction('p2', 'placeToken', {
        target: '/zones/board/cells/1/1',
        entity: 'piece_p2'
      });
      await this.executeAction('p1', 'placeToken', {
        target: '/zones/board/cells/0/1',
        entity: 'piece_p1'
      });
      await this.executeAction('p2', 'placeToken', {
        target: '/zones/board/cells/2/0',
        entity: 'piece_p2'
      });
      await this.executeAction('p1', 'placeToken', {
        target: '/zones/board/cells/0/2',
        entity: 'piece_p1'
      });
      
      // P1 now has 3 pieces, verify this
      assert.strictEqual(this.pieceCounts.p1, 3, 'P1 should have 3 pieces before attempting 4th');
      
      // Try to place 4th piece - should fail
      const piecesBeforeP1 = this.pieceCounts.p1;
      
      await this.executeAction('p1', 'placeToken', {
        target: '/zones/board/cells/1/0',
        entity: 'piece_p1'
      });
      
      assert.strictEqual(this.pieceCounts.p1, piecesBeforeP1, 'P1 should not place 4th piece');
    });

    await this.testScenario('Cannot Place on Occupied Space', async () => {
      // Reset and place one piece
      await this.resetGame();
      await this.executeAction('p1', 'placeToken', {
        target: '/zones/board/cells/0/0',
        entity: 'piece_p1'
      });
      
      // P2 tries same spot
      const boardBefore = JSON.stringify(this.board);
      await this.executeAction('p2', 'placeToken', {
        target: '/zones/board/cells/0/0',
        entity: 'piece_p2'
      });
      
      const boardAfter = JSON.stringify(this.board);
      assert.strictEqual(boardBefore, boardAfter, 'Board should not change');
      assert.strictEqual(this.currentPlayer, 'p2', 'Turn should not advance');
    });
  }

  async testMovementPhase() {
    console.log('\n🏃 Testing Movement Phase\n');
    
    await this.testScenario('Select and Move Piece', async () => {
      // Setup board in movement phase
      await this.setupMovementPhase();
      
      // P1 selects a piece
      await this.executeAction('p1', 'selectPiece', {
        target: '/zones/board/cells/0/0',
        player: 'p1'
      });
      
      assert(this.selectedPiece, 'Should have selected piece');
      
      // Move selected piece
      await this.executeAction('p1', 'moveSelectedPiece', {
        target: '/zones/board/cells/1/0',
        player: 'p1'
      });
      
      assert.strictEqual(this.board[0][0], null, 'Original position should be empty');
      assert.strictEqual(this.board[1][0], 'piece_p1', 'Piece should be at new position');
      assert.strictEqual(this.currentPlayer, 'p2', 'Turn should advance');
    });

    await this.testScenario('Cancel Selection', async () => {
      // Setup and select piece
      await this.setupMovementPhase();
      await this.executeAction('p1', 'selectPiece', {
        target: '/zones/board/cells/0/0',
        player: 'p1'
      });
      
      assert(this.selectedPiece, 'Should have selected piece');
      
      // Cancel selection
      await this.executeAction('p1', 'clearSelection', {
        player: 'p1'
      });
      
      assert.strictEqual(this.selectedPiece, null, 'Selection should be cleared');
      assert.strictEqual(this.currentPlayer, 'p1', 'Turn should not advance on cancel');
    });

    await this.testScenario('Cannot Select Opponent Piece', async () => {
      await this.setupMovementPhase();
      
      // P1 tries to select P2's piece
      const selectionBefore = this.selectedPiece;
      await this.executeAction('p1', 'selectPiece', {
        target: '/zones/board/cells/1/1',
        player: 'p1'
      });
      
      assert.strictEqual(this.selectedPiece, selectionBefore, 'Should not select opponent piece');
    });

    await this.testScenario('Cannot Move to Occupied Space', async () => {
      await this.setupMovementPhase();
      
      // Select piece
      await this.executeAction('p1', 'selectPiece', {
        target: '/zones/board/cells/0/0',
        player: 'p1'
      });
      
      // Try to move to occupied space
      const boardBefore = JSON.stringify(this.board);
      await this.executeAction('p1', 'moveSelectedPiece', {
        target: '/zones/board/cells/1/1', // Occupied by P2
        player: 'p1'
      });
      
      const boardAfter = JSON.stringify(this.board);
      assert.strictEqual(boardBefore, boardAfter, 'Board should not change');
      assert(this.selectedPiece, 'Piece should still be selected');
    });
  }

  async testWinConditions() {
    console.log('\n🏆 Testing Win Conditions\n');
    
    await this.testScenario('Win During Placement - Horizontal', async () => {
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 0 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 0, col: 2 }, // P1 wins with top row
      ];
      
      await this.playPlacementMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win horizontally');
    });

    await this.testScenario('Win During Placement - Vertical', async () => {
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 0, col: 1 },
        { player: 'p1', row: 1, col: 0 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 2, col: 0 }, // P1 wins with left column
      ];
      
      await this.playPlacementMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win vertically');
    });

    await this.testScenario('Win During Placement - Diagonal', async () => {
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 0, col: 1 },
        { player: 'p1', row: 1, col: 1 },
        { player: 'p2', row: 0, col: 2 },
        { player: 'p1', row: 2, col: 2 }, // P1 wins diagonally
      ];
      
      await this.playPlacementMoves(moves);
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win diagonally');
    });

    await this.testScenario('Win During Movement Phase', async () => {
      // Setup non-winning position
      const setupMoves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 2, col: 0 },
        { player: 'p1', row: 1, col: 2 },
        { player: 'p2', row: 2, col: 2 },
      ];
      
      await this.playPlacementMoves(setupMoves);
      assert.strictEqual(this.currentPhase, 'movement', 'Should be in movement phase');
      
      // P1 moves to complete a line
      await this.executeAction('p1', 'selectPiece', {
        target: '/zones/board/cells/1/2',
        player: 'p1'
      });
      
      await this.executeAction('p1', 'moveSelectedPiece', {
        target: '/zones/board/cells/0/2', // Complete top row
        player: 'p1'
      });
      
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 should win after movement');
    });
  }

  async testInvalidMoves() {
    console.log('\n❌ Testing Invalid Moves\n');
    
    await this.testScenario('Out of Turn - Placement', async () => {
      // P1 places
      await this.executeAction('p1', 'placeToken', {
        target: '/zones/board/cells/0/0',
        entity: 'piece_p1'
      });
      
      // P1 tries again
      const boardBefore = JSON.stringify(this.board);
      await this.executeAction('p1', 'placeToken', {
        target: '/zones/board/cells/0/1',
        entity: 'piece_p1'
      });
      
      const boardAfter = JSON.stringify(this.board);
      assert.strictEqual(boardBefore, boardAfter, 'Board should not change');
      assert.strictEqual(this.currentPlayer, 'p2', 'Should be P2 turn');
    });

    await this.testScenario('Out of Turn - Movement', async () => {
      await this.setupMovementPhase();
      
      // P1 selects and moves
      await this.executeAction('p1', 'selectPiece', {
        target: '/zones/board/cells/0/0',
        player: 'p1'
      });
      await this.executeAction('p1', 'moveSelectedPiece', {
        target: '/zones/board/cells/1/0',
        player: 'p1'
      });
      
      // P1 tries to go again
      const boardBefore = JSON.stringify(this.board);
      await this.executeAction('p1', 'selectPiece', {
        target: '/zones/board/cells/0/1',
        player: 'p1'
      });
      
      assert.strictEqual(this.selectedPiece, null, 'Should not select when not turn');
    });
  }

  async testEdgeCases() {
    console.log('\n🔧 Testing Edge Cases\n');
    
    await this.testScenario('Move After Game End', async () => {
      // Create winning position
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 0 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 0, col: 2 }, // P1 wins
      ];
      
      await this.playPlacementMoves(moves);
      assert(this.gameEnded, 'Game should be ended');
      
      // Try to place after win
      const boardBefore = JSON.stringify(this.board);
      await this.executeAction('p2', 'placeToken', {
        target: '/zones/board/cells/2/2',
        entity: 'piece_p2'
      });
      
      const boardAfter = JSON.stringify(this.board);
      assert.strictEqual(boardBefore, boardAfter, 'No moves after game end');
    });

    await this.testScenario('All Corners Occupied', async () => {
      // Place pieces in all corners
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 0, col: 2 },
        { player: 'p1', row: 2, col: 0 },
        { player: 'p2', row: 2, col: 2 },
        { player: 'p1', row: 1, col: 1 },
        { player: 'p2', row: 0, col: 1 },
      ];
      
      await this.playPlacementMoves(moves);
      assert.strictEqual(this.currentPhase, 'movement', 'Should handle corner-heavy placement');
    });

    await this.testScenario('Minimum Moves to Win', async () => {
      // Fastest possible win is 5 moves
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 1, col: 0 },
        { player: 'p1', row: 0, col: 1 },
        { player: 'p2', row: 1, col: 1 },
        { player: 'p1', row: 0, col: 2 }, // Win in 5 moves
      ];
      
      await this.playPlacementMoves(moves);
      assert.strictEqual(this.pieceCounts.p1, 3, 'Minimum 3 pieces to win');
      assert.strictEqual(this.gameStatus?.winner, 'p1', 'P1 wins');
    });
  }

  // Helper methods
  async setupMovementPhase() {
    // Reset and place all pieces without winning
    await this.resetGame();
    
    const setupMoves = [
      { player: 'p1', row: 0, col: 0 },
      { player: 'p2', row: 1, col: 1 },
      { player: 'p1', row: 0, col: 1 },
      { player: 'p2', row: 2, col: 0 },
      { player: 'p1', row: 1, col: 2 },
      { player: 'p2', row: 2, col: 2 },
    ];
    
    await this.playPlacementMoves(setupMoves);
  }

  async playPlacementMoves(moves) {
    for (const move of moves) {
      if (this.gameEnded) break;
      if (this.currentPlayer === move.player) {
        await this.executeAction(move.player, 'placeToken', {
          target: `/zones/board/cells/${move.row}/${move.col}`,
          entity: `piece_${move.player}`
        });
      }
    }
  }

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
    this.board = Array(3).fill(null).map(() => Array(3).fill(null));
    this.pieceCounts = { p1: 0, p2: 0 };
    this.currentPhase = 'placement';
    this.selectedPiece = null;
    this.gameEnded = false;
    this.gameStatus = null;
    this.currentPlayer = null;
    
    // Create new game
    await this.createLobby();
    await this.connectPlayers(['Alice', 'Bob']);
    await this.startGame();
  }

  printBoard() {
    console.log('\nBoard:');
    this.board.forEach((row, rowIdx) => {
      const rowStr = row.map(cell => {
        if (!cell) return '·';
        return cell.includes('p1') ? '⚪' : '⚫';
      }).join(' ');
      console.log(`  ${rowStr}`);
    });
  }

  // Main test runner
  async runAllTests() {
    console.log('\n🎮 THREE MEN\'S MORRIS REGRESSION TEST SUITE\n');
    
    try {
      await this.testPlacementPhase();
      await this.testMovementPhase();
      await this.testWinConditions();
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
  const test = new ThreeMensMorrisRegressionTest();
  
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

module.exports = { ThreeMensMorrisRegressionTest };

if (require.main === module) {
  runTest();
}