const WebSocket = require('ws');
const assert = require('assert');
const { GameLogValidator } = require('./GameLogValidator');

/**
 * Reusable test framework for Bluefelt games
 */
class GameTestFramework {
  constructor(gameId) {
    this.gameId = gameId;
    this.apiUrl = 'http://localhost:8000/api';
    this.wsUrl = 'ws://localhost:8000/api/lobbies';
    
    // Game state
    this.lobbyId = null;
    this.players = new Map();
    this.gameState = null;
    this.messageLog = [];
    this.currentPhase = null;
    this.currentPlayer = null;
    this.gameEnded = false;
    
    // Test state
    this.assertions = [];
    this.lastTick = -1;
    
    // Game logs
    this.gameLogs = [];
    this.logValidator = new GameLogValidator();
  }
  
  // Lobby management
  async createLobby(seed = null) {
    const requestBody = { game_id: this.gameId };
    if (seed) {
      requestBody.seed = seed;
      console.log(`  Using fixed seed: ${seed}`);
    }
    
    const response = await fetch(`${this.apiUrl}/lobbies`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(requestBody)
    });
    
    const lobby = await response.json();
    this.lobbyId = lobby.id;
    console.log(`✓ Lobby created: ${lobby.id}`);
    return lobby;
  }
  
  // Player management
  async connectPlayer(name) {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(`${this.wsUrl}/${this.lobbyId}/ws?player=${name}&join=true`);
      
      ws.on('open', () => {
        console.log(`✓ ${name} connected`);
      });
      
      ws.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        this.handleMessage(ws, name, msg);
      });
      
      ws.on('error', reject);
      
      resolve(ws);
    });
  }
  
  async connectPlayers(names) {
    const connections = [];
    for (const name of names) {
      const ws = await this.connectPlayer(name);
      connections.push(ws);
      await this.wait(200);
    }
    return connections;
  }
  
  // Message handling
  handleMessage(ws, playerName, msg) {
    // Log all messages for debugging
    this.messageLog.push({ player: playerName, type: msg.type, tick: msg.tick });
    
    switch (msg.type) {
      case 'welcome':
        this.handleWelcome(ws, playerName, msg);
        break;
        
      case 'gameStarted':
        this.handleGameStarted(msg);
        break;
        
      case 'diff':
        // Only process diffs once per tick
        if (msg.tick !== undefined && msg.tick !== this.lastTick) {
          this.lastTick = msg.tick;
          this.handleDiff(msg);
        }
        break;
        
      case 'error':
        console.error(`❌ Server error: ${msg.message}`);
        this.gameEnded = true;
        break;
    }
  }
  
  handleWelcome(ws, playerName, msg) {
    const playerId = msg.you;
    this.players.set(playerId, { name: playerName, id: playerId, ws });
    console.log(`  ${playerName} = ${playerId}`);
  }
  
  handleGameStarted(msg) {
    console.log(`\n=== GAME STARTED ===`);
    this.gameState = msg.game;
    this.currentPlayer = msg.game?.currentPlayer;
    this.currentPhase = msg.game?.phases?.game;
    this.lastTick = msg.game?.tick || 0;  // Initialize tick from game state
    
    // Let subclasses handle game-specific setup
    this.onGameStarted(msg);
  }
  
  handleDiff(msg) {
    console.log(`\n[Tick ${msg.tick}]`);
    
    // Extract game logs from patches
    const logPatches = msg.patch.filter(p => p.path.startsWith('/ui/gameLog/'));
    for (const logPatch of logPatches) {
      if (logPatch.op === 'add' && logPatch.value?.message) {
        this.gameLogs.push(logPatch.value.message);
        this.logValidator.addLog(logPatch.value.message);
        console.log(`  📝 Log: ${logPatch.value.message}`);
      }
    }
    
    // Let subclasses process patches first
    this.processPatch(msg.patch);
    
    // Check for game end
    const gameStatusPatch = msg.patch.find(p => p.path === '/game/gameStatus');
    if (gameStatusPatch && gameStatusPatch.value?.state === 'ended') {
      this.gameEnded = true;
      this.onGameEnded(gameStatusPatch.value);
    }
  }
  
  // Actions
  async startGame() {
    const firstPlayer = Array.from(this.players.values())[0];
    console.log(`\nStarting game...`);
    firstPlayer.ws.send(JSON.stringify({ action: 'start_game' }));
    await this.wait(1000);
  }
  
  async executeAction(playerId, action, args) {
    const player = this.players.get(playerId);
    if (!player) {
      throw new Error(`Player ${playerId} not found`);
    }
    
    console.log(`\n${player.name} executes: ${action}`);
    console.log(`  Args: ${JSON.stringify(args)}`);
    
    return new Promise((resolve) => {
      const startTick = this.lastTick;
      player.ws.send(JSON.stringify({ action, args }));
      
      // Wait for next tick
      const checkInterval = setInterval(() => {
        if (this.lastTick > startTick || this.gameEnded) {
          clearInterval(checkInterval);
          resolve();
        }
      }, 50);
      
      // Timeout after 3 seconds
      setTimeout(() => {
        clearInterval(checkInterval);
        console.log('  ⚠️  Action timed out');
        resolve();
      }, 3000);
    });
  }
  
  // Utilities
  async wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  getCurrentPlayer() {
    return this.players.get(this.currentPlayer);
  }
  
  // Assertions
  addAssertion(description, fn) {
    this.assertions.push({ description, fn });
  }
  
  async runAssertions() {
    console.log('\n=== RUNNING ASSERTIONS ===');
    let passed = 0;
    let failed = 0;
    
    for (const assertion of this.assertions) {
      try {
        await assertion.fn();
        console.log(`✓ ${assertion.description}`);
        passed++;
      } catch (error) {
        console.error(`✗ ${assertion.description}`);
        console.error(`  ${error.message}`);
        failed++;
      }
    }
    
    console.log(`\nResults: ${passed} passed, ${failed} failed`);
    return failed === 0;
  }
  
  // Log validation methods
  expectLog(pattern, description) {
    this.logValidator.expectLog(pattern, description);
  }
  
  expectNoLog(pattern, description) {
    this.logValidator.expectNoLog(pattern, description);
  }
  
  expectLogOrder(patterns, description) {
    this.logValidator.expectLogOrder(patterns, description);
  }
  
  validateLogs() {
    const results = this.logValidator.validate();
    return this.logValidator.printResults(results);
  }
  
  // Cleanup
  cleanup() {
    this.players.forEach(player => {
      if (player.ws.readyState === WebSocket.OPEN) {
        player.ws.close();
      }
    });
  }
  
  // Override these in subclasses
  onGameStarted(msg) {}
  processPatch(patches) {}
  onGameEnded(status) {}
}

// Example: Tic-Tac-Toe test
class TicTacToeTest extends GameTestFramework {
  constructor() {
    super('tic-tac-toe');
    this.board = Array(3).fill(null).map(() => Array(3).fill(null));
    this.moveCount = 0;
  }
  
  onGameStarted(msg) {
    if (msg.game?.zones?.board?.cells) {
      this.board = msg.game.zones.board.cells;
    }
  }
  
  processPatch(patches) {
    for (const patch of patches) {
      // Board updates
      if (patch.path.match(/\/game\/zones\/board\/cells\/(\d+)\/(\d+)/)) {
        const [, row, col] = patch.path.match(/\/(\d+)\/(\d+)$/);
        const entity = patch.value?.entity;
        this.board[parseInt(row)][parseInt(col)] = entity;
        this.moveCount++;
        console.log(`  Move ${this.moveCount}: ${this.currentPlayer} → (${row},${col})`);
      }
      
      // Turn updates
      if (patch.path === '/game/currentPlayer') {
        this.currentPlayer = patch.value;
      }
    }
  }
  
  onGameEnded(status) {
    if (status.winner) {
      console.log(`\n🏆 Winner: ${status.winner}!`);
    } else if (status.tie) {
      console.log(`\n🤝 Game ended in a tie!`);
    }
  }
  
  printBoard() {
    console.log('\nBoard:');
    this.board.forEach(row => {
      const rowStr = row.map(cell => {
        if (!cell) return '·';
        return cell.includes('p1') ? 'X' : 'O';
      }).join(' ');
      console.log(`  ${rowStr}`);
    });
  }
}

// Export for use in other tests
module.exports = { GameTestFramework, TicTacToeTest };

// Run example test if executed directly
if (require.main === module) {
  async function runTicTacToeTest() {
    const test = new TicTacToeTest();
    
    try {
      // Setup
      await test.createLobby();
      await test.connectPlayers(['Alice', 'Bob']);
      await test.startGame();
      
      // Play winning game
      const moves = [
        { player: 'p1', row: 0, col: 0 },
        { player: 'p2', row: 0, col: 1 },
        { player: 'p1', row: 1, col: 1 },
        { player: 'p2', row: 0, col: 2 },
        { player: 'p1', row: 2, col: 2 },
      ];
      
      for (const move of moves) {
        if (test.gameEnded) break;
        
        if (test.currentPlayer === move.player) {
          await test.executeAction(move.player, 'placeMarker', {
            target: `/zones/board/cells/${move.row}/${move.col}`
          });
        }
      }
      
      test.printBoard();
      
      // Assertions
      test.addAssertion('Game should end', () => {
        assert(test.gameEnded);
      });
      
      test.addAssertion('P1 should win', () => {
        assert(test.moveCount === 5);
      });
      
      const success = await test.runAssertions();
      test.cleanup();
      
      process.exit(success ? 0 : 1);
      
    } catch (error) {
      console.error('Test failed:', error);
      test.cleanup();
      process.exit(1);
    }
  }
  
  runTicTacToeTest();
}