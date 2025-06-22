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
    this.gameStarted = false;
    this.actionMap = null;
    this.uiData = null;
    
    // Test state
    this.assertions = [];
    this.lastTick = -1;
    this.lastMoveTickProcessed = -1;
    this.patches = [];
    
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
    
    // Debug logging (reduced to prevent EPIPE errors)
    if (msg.type === 'gameStarted' || msg.type === 'tableCreated' || msg.type === 'tableUpdated' || msg.type === 'error') {
      console.log(`  DEBUG: ${playerName} received ${msg.type} message`);
    }
    
    switch (msg.type) {
      case 'welcome':
        this.handleWelcome(ws, playerName, msg);
        // If game has started and we got a proper actor ID, update mappings
        if (msg.you && msg.you !== null) {
          const playerData = this.players.get(playerName);
          if (playerData) {
            playerData.id = msg.you;
            playerData.welcomeMessage = msg;  // Store the updated welcome message
            this.players.set(msg.you, playerData);
            console.log(`  Updated ${playerName} → ${msg.you} (from welcome)`);
          }
        }
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
        
      case 'lobbyJoined':
        // Handle new lobby system messages
        this.handleLobbyJoined(ws, playerName, msg);
        break;
        
      case 'lobbyState':
        // Handle lobby state updates  
        this.handleLobbyState(msg);
        break;
        
      case 'tableCreated':
        // Handle table creation
        this.handleTableCreated(msg);
        break;
        
      case 'tableUpdated':
        // Handle table updates (like players joining)
        console.log(`  ✓ Table updated`);
        break;
        
      case 'tableJoined':
        // Handle player joining table
        console.log(`  ✓ ${playerName} joined table`);
        break;
        
      case 'gameJoined':
        // Handle game joining (for compatibility)
        console.log(`  ✓ ${playerName} joined game`);
        break;
        
      case 'error':
        console.error(`❌ Server error: ${msg.message}`);
        this.gameEnded = true;
        break;
        
      case 'state':
        this.handleState(msg);
        break;
        
      case 'patch':
        this.handlePatch(msg);
        break;
        
      case 'gameUpdate':
        // Handle new system's game updates
        if (msg.ui) {
          this.actionMap = msg.ui.actionMap;
          this.uiData = msg.ui;
        }
        if (msg.patches) {
          this.handleDiff({
            tick: msg.tick || this.lastTick + 1,
            patch: msg.patches
          });
        }
        break;
    }
  }
  
  handleWelcome(ws, playerName, msg) {
    const playerId = msg.you;
    // Store by player name initially since actor ID might not be assigned yet
    this.players.set(playerName, { name: playerName, id: playerId, ws, welcomeMessage: msg });
    console.log(`  ${playerName} = ${playerId}`);
  }
  
  handleLobbyJoined(ws, playerName, msg) {
    // For new lobby system, store WebSocket connection
    this.players.set(playerName, { name: playerName, id: playerName, ws, lobbyMessage: msg });
    this.lobbyState = msg.lobby;
  }
  
  handleLobbyState(msg) {
    // Update lobby state
    this.lobbyState = msg.lobby;
  }
  
  handleTableCreated(msg) {
    // Initialize tables array if it doesn't exist
    if (!this.lobbyState) {
      this.lobbyState = { tables: [] };
    }
    if (!this.lobbyState.tables) {
      this.lobbyState.tables = [];
    }
    
    // Add the created table to the lobby state (avoid duplicates)
    if (msg.table) {
      const existingTable = this.lobbyState.tables.find(t => t.id === msg.table.id);
      if (!existingTable) {
        this.lobbyState.tables.push(msg.table);
        console.log(`  ✓ Table created: ${msg.table.id}`);
      }
    }
  }
  
  handleGameStarted(msg) {
    console.log(`\n=== GAME STARTED ===`);
    
    // Handle both old format (msg.game) and new format (msg.state)
    this.gameState = msg.state || msg.game;
    this.currentPlayer = this.gameState?.currentPlayer;
    this.currentPhase = this.gameState?.phases?.game;
    this.lastTick = msg.tick || this.gameState?.tick || 0;
    this.gameStarted = true;
    
    // Update table status in lobby state to Playing
    // Note: gameInstanceId corresponds to the table ID
    if (msg.gameInstanceId && this.lobbyState?.tables) {
      const tableIndex = this.lobbyState.tables.findIndex(t => t.id === msg.gameInstanceId);
      if (tableIndex >= 0) {
        this.lobbyState.tables[tableIndex].status = 'Playing';
        console.log(`  ✓ Updated table ${msg.gameInstanceId} status to Playing`);
      }
    }
    
    // Store UI data including action map
    if (msg.ui) {
      this.actionMap = msg.ui.actionMap;
      this.uiData = msg.ui;
    }
    
    // Update player mappings
    if (msg.players) {
      // New format: msg.players is a mapping like {"p1": "Alice", "p2": "Bob"}
      for (const [slot, playerName] of Object.entries(msg.players)) {
        if (this.players.has(playerName)) {
          const playerData = this.players.get(playerName);
          playerData.id = slot;
          // Also store by slot ID for easy lookup
          this.players.set(slot, playerData);
          console.log(`  Updated ${playerName} → ${slot}`);
        }
      }
    } else if (this.gameState?.players) {
      // Old format: game.players is an array
      this.gameState.players.forEach((player, index) => {
        const playerName = player.name;
        const playerId = player.id;
        if (this.players.has(playerName)) {
          const playerData = this.players.get(playerName);
          playerData.id = playerId;
          // Also store by actor ID for easy lookup
          this.players.set(playerId, playerData);
          console.log(`  Updated ${playerName} → ${playerId}`);
        }
      });
    }
    
    // Let subclasses handle game-specific setup
    this.onGameStarted(msg);
  }
  
  handleDiff(msg) {
    console.log(`\n[Tick ${msg.tick}]`);
    
    // Store patches for later analysis
    if (msg.patch) {
      this.patches.push(...msg.patch);
    }
    
    // Extract game logs from patches
    const logPatches = msg.patch.filter(p => p.path.startsWith('/ui/gameLog/'));
    for (const logPatch of logPatches) {
      if (logPatch.op === 'add' && logPatch.value?.message) {
        this.gameLogs.push(logPatch.value.message);
        this.logValidator.addLog(logPatch.value.message);
        console.log(`  📝 Log: ${logPatch.value.message}`);
      }
    }
    
    // Apply UI patches to maintain UI state
    for (const patch of msg.patch) {
      if (patch.path.startsWith('/ui/')) {
        // Initialize uiData if needed
        if (!this.uiData) this.uiData = {};
        
        // Apply patch to UI data
        const pathParts = patch.path.substring(4).split('/').filter(p => p);
        if (pathParts.length > 0) {
          let target = this.uiData;
          for (let i = 0; i < pathParts.length - 1; i++) {
            if (!target[pathParts[i]]) {
              target[pathParts[i]] = {};
            }
            target = target[pathParts[i]];
          }
          
          if (patch.op === 'replace' || patch.op === 'add') {
            target[pathParts[pathParts.length - 1]] = patch.value;
          } else if (patch.op === 'remove') {
            delete target[pathParts[pathParts.length - 1]];
          }
        }
      }
      
      // Also apply game state patches
      if (patch.path.startsWith('/game/')) {
        // Initialize gameState if needed
        if (!this.gameState) this.gameState = {};
        
        // Apply patch to game state
        const pathParts = patch.path.substring(6).split('/').filter(p => p);
        if (pathParts.length > 0) {
          let target = this.gameState;
          for (let i = 0; i < pathParts.length - 1; i++) {
            if (!target[pathParts[i]]) {
              target[pathParts[i]] = {};
            }
            target = target[pathParts[i]];
          }
          
          if (patch.op === 'replace' || patch.op === 'add') {
            target[pathParts[pathParts.length - 1]] = patch.value;
          } else if (patch.op === 'remove') {
            delete target[pathParts[pathParts.length - 1]];
          }
        }
        
        // Update currentPlayer if patched
        if (patch.path === '/game/currentPlayer') {
          this.currentPlayer = patch.value;
        }
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
  
  handleState(msg) {
    console.log(`\n=== STATE UPDATE ===`);
    // Modern servers send full state in 'state' messages
    if (msg.state) {
      this.gameState = msg.state;
      
      // Update tracking variables
      if (msg.state.currentPlayer) {
        this.currentPlayer = msg.state.currentPlayer;
      }
      
      // Handle both old and enhanced phase systems
      if (msg.state.phases) {
        this.currentPhase = msg.state.phases.game || 
          (msg.state.phases.current && msg.state.phases.current.game) || 
          this.currentPhase;
      }
      
      // Update tick
      if (msg.state.tick !== undefined) {
        this.lastTick = msg.state.tick;
      }
      
      // Store UI data including action map
      if (msg.ui) {
        this.actionMap = msg.ui.actionMap;
        this.uiData = msg.ui;
      }
      
      // Apply UI patches from diff to maintain UI state
      if (msg.patch) {
        for (const patch of msg.patch) {
          if (patch.path.startsWith('/ui/')) {
            // Initialize uiData if needed
            if (!this.uiData) this.uiData = {};
            
            // Apply patch to UI data
            const pathParts = patch.path.substring(4).split('/').filter(p => p);
            if (pathParts.length > 0) {
              let target = this.uiData;
              for (let i = 0; i < pathParts.length - 1; i++) {
                if (!target[pathParts[i]]) {
                  target[pathParts[i]] = {};
                }
                target = target[pathParts[i]];
              }
              
              if (patch.op === 'replace' || patch.op === 'add') {
                target[pathParts[pathParts.length - 1]] = patch.value;
              } else if (patch.op === 'remove') {
                delete target[pathParts[pathParts.length - 1]];
              }
            }
          }
        }
      }
      
      // If this is the initial state, treat it like gameStarted
      if (!this.gameStarted && msg.state.players) {
        this.gameStarted = true;
        this.onGameStarted({ game: msg.state, ui: msg.ui });
      }
    }
  }
  
  handlePatch(msg) {
    console.log(`\n[Patch - Tick ${msg.tick || 'unknown'}]`);
    
    // Modern servers send patches directly
    if (msg.patches) {
      // Extract game logs from patches
      const logPatches = msg.patches.filter(p => p.path.startsWith('/ui/gameLog/'));
      for (const logPatch of logPatches) {
        if (logPatch.op === 'add' && logPatch.value?.message) {
          this.gameLogs.push(logPatch.value.message);
          this.logValidator.addLog(logPatch.value.message);
          console.log(`  📝 Log: ${logPatch.value.message}`);
        }
      }
      
      // Let subclasses process patches
      this.processPatch(msg.patches);
      
      // Check for game end
      const gameStatusPatch = msg.patches.find(p => 
        p.path === '/game/gameStatus' || p.path === '/gameStatus'
      );
      if (gameStatusPatch && gameStatusPatch.value?.state === 'ended') {
        this.gameEnded = true;
        this.onGameEnded(gameStatusPatch.value);
      }
      
      // Update tick if provided
      if (msg.tick !== undefined) {
        this.lastTick = msg.tick;
      }
    }
  }
  
  // Actions
  async startGame() {
    const players = Array.from(this.players.values());
    const firstPlayer = players[0];
    
    if (!firstPlayer || !firstPlayer.ws) {
      throw new Error('No players connected with WebSocket');
    }
    
    console.log(`\nStarting game...`);
    
    // For new table system, create a table and auto-join players
    // Create a table
    firstPlayer.ws.send(JSON.stringify({ 
      action: 'createTable', 
      bundleId: this.gameId 
    }));
    await this.wait(500);
    
    // Find the created table ID from lobby state
    const table = this.lobbyState?.tables?.[0];
    if (!table) {
      throw new Error('No table created');
    }
    
    // Have all players except the table creator join the table (auto-seat assignment)
    // Note: The table creator is automatically seated when creating the table
    for (let i = 1; i < players.length; i++) {
      const player = players[i];
      player.ws.send(JSON.stringify({ 
        action: 'joinTable', 
        tableId: table.id 
      }));
      await this.wait(200);
    }
    
    // All players mark ready
    for (const player of players) {
      player.ws.send(JSON.stringify({ 
        action: 'setReady', 
        tableId: table.id,
        ready: true
      }));
      await this.wait(100);
    }
    
    // Wait for countdown and game start (countdown is 10 seconds, add buffer)
    await this.wait(12000);
  }
  
  async executeAction(playerId, action, args) {
    // Ensure playerId is a string (fix for [object Object] bug)
    const playerIdStr = typeof playerId === 'object' ? JSON.stringify(playerId) : String(playerId);
    
    const player = this.players.get(playerIdStr);
    if (!player) {
      throw new Error(`Player ${playerIdStr} not found`);
    }
    
    console.log(`\n${player.name} executes: ${action}`);
    console.log(`  Args: ${JSON.stringify(args)}`);
    
    // For new table system, need to send gameAction with the table ID that has an active game
    const table = this.lobbyState?.tables?.find(t => t.status === 'Playing');
    if (!table) {
      // Fallback: try to find any table that exists
      const anyTable = this.lobbyState?.tables?.[0];
      if (anyTable) {
        return await this.executeActionWithTable(player, action, args, anyTable);
      }
      throw new Error(`No active game table found. Available: ${JSON.stringify(this.lobbyState?.tables?.map(t => ({ id: t.id, status: t.status })) || 'none')}`);
    }
    
    return await this.executeActionWithTable(player, action, args, table);
  }
  
  executeActionWithTable(player, action, args, table) {
    return new Promise((resolve) => {
      const startTime = Date.now();
      
      // Send the action with table ID as gameId (for compatibility with existing gameAction handler)
      player.ws.send(JSON.stringify({ 
        action: 'gameAction',
        gameId: table.id,  // Use table ID as gameId since that's where the game is running
        data: { action, ...args }
      }));
      
      // Wait a reasonable time for the action to be processed
      // This lets the main handleMessage process all patches properly
      setTimeout(() => {
        resolve();
      }, 200);
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
  async cleanup() {
    console.log('\n🧹 Cleaning up test resources...');
    
    // Close all WebSocket connections more robustly
    const closePromises = [];
    this.players.forEach(player => {
      if (player.ws) {
        if (player.ws.readyState === WebSocket.OPEN || player.ws.readyState === WebSocket.CONNECTING) {
          // Remove all listeners to prevent memory leaks
          player.ws.onopen = null;
          player.ws.onmessage = null;
          player.ws.onerror = null;
          player.ws.onclose = null;
          
          // Create a promise that resolves when the connection closes
          const closePromise = new Promise((resolve) => {
            player.ws.onclose = resolve;
            // Timeout after 2 seconds if close doesn't fire
            setTimeout(resolve, 2000);
          });
          closePromises.push(closePromise);
          
          // Close the connection
          player.ws.close();
          console.log(`  ✓ Closed WebSocket for ${player.name}`);
        }
      }
    });
    
    // Wait for all connections to close
    await Promise.all(closePromises);
    
    // Clear all state
    this.players.clear();
    this.gameState = null;
    this.actionMap = null;
    this.uiData = null;
    this.gameEnded = false;
    this.gameStarted = false;
    this.lastTick = -1;
    this.lastMoveTickProcessed = -1;
    
    // Delete the lobby via HTTP API if we have a lobby ID
    if (this.lobbyId) {
      try {
        const response = await fetch(`http://localhost:8000/api/lobbies/${this.lobbyId}`, {
          method: 'DELETE'
        });
        if (response.ok) {
          console.log(`  ✓ Deleted lobby ${this.lobbyId}`);
        } else if (response.status === 404) {
          // Lobby already gone, that's fine
          console.log(`  ✓ Lobby ${this.lobbyId} already deleted`);
        } else {
          console.log(`  ⚠️  Failed to delete lobby ${this.lobbyId}: ${response.status}`);
        }
      } catch (error) {
        console.log(`  ⚠️  Error deleting lobby: ${error.message}`);
      }
    }
    
    // Wait a bit to ensure cleanup is complete
    await this.wait(500);
    
    console.log('  ✓ Cleanup complete');
  }
  
  // Additional helper methods for regression tests
  
  // Get recent patches
  getRecentPatches(count = 10) {
    return this.patches.slice(-count);
  }
  
  // Get current board state
  async getBoardState() {
    if (this.gameState?.zones?.board?.cells) {
      return this.gameState.zones.board.cells;
    }
    return null;
  }
  
  // Get multi-step state for a player
  async getMultiStepState(playerId) {
    return this.uiData?.multiStepState?.[playerId] || null;
  }
  
  // Send action using player name
  async sendAction(playerName, action, args = {}) {
    const player = this.players.get(playerName);
    if (!player) {
      throw new Error(`Player ${playerName} not found`);
    }
    
    console.log(`${playerName} sends: ${action} ${JSON.stringify(args)}`);
    player.ws.send(JSON.stringify({ action, args }));
  }
  
  // Wait for state update
  async waitForStateUpdate(timeoutMs = 2000) {
    const startTick = this.lastTick;
    const startTime = Date.now();
    
    while (this.lastTick === startTick && Date.now() - startTime < timeoutMs) {
      await this.wait(50);
    }
    
    if (this.lastTick === startTick) {
      console.log('⚠️  Timeout waiting for state update');
    }
  }
  
  // Get current game phase
  async getGamePhase() {
    return this.currentPhase;
  }
  
  // Print board state (generic)
  printBoard(board) {
    if (!board) return;
    
    board.forEach((row, rowIdx) => {
      const rowStr = row.map(cell => {
        if (!cell || !cell.entity) return '·';
        // Extract player number from entity ID
        const match = cell.entity.match(/p(\d+)$/);
        return match ? `p${match[1]}` : cell.entity;
      }).join(' ');
      console.log(`  ${rowStr}`);
    });
  }
  
  // Simple assertion helper
  assert(condition, message) {
    if (!condition) {
      throw new Error(message || 'Assertion failed');
    }
  }
  
  // Connect multiple players
  async connect(playerNames) {
    for (const name of playerNames) {
      await this.connectPlayer(name);
      await this.wait(200);
    }
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
    // Track if this patch batch contains a board update  
    let boardUpdated = false;
    let lastBoardUpdate = null;
    
    for (const patch of patches) {
      // Board updates (matches cells in any board-like zone)
      if (patch.path.match(/\/game\/zones\/\w+\/cells\/(\d+)\/(\d+)/)) {
        const [, row, col] = patch.path.match(/\/(\d+)\/(\d+)$/);
        const entity = patch.value?.entity;
        
        // Update internal board tracking
        if (!this.board[parseInt(row)]) {
          this.board[parseInt(row)] = [];
        }
        this.board[parseInt(row)][parseInt(col)] = entity;
        
        // Only track if this is a new entity placement (not clearing)
        if (entity && !entity.includes('empty')) {
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
    
    // Only increment move count once per patch batch with board updates
    // and only if we haven't already counted this tick
    if (boardUpdated && lastBoardUpdate && this.lastTick !== this.lastMoveTickProcessed) {
      this.moveCount++;
      this.lastMoveTickProcessed = this.lastTick;
      console.log(`  Move ${this.moveCount}: ${lastBoardUpdate.entity} → (${lastBoardUpdate.row},${lastBoardUpdate.col})`);
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