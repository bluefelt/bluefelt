#!/usr/bin/env node

/**
 * Simple game validation script to test all games in games/ directory
 * Focuses on basic functionality rather than complex scenarios
 */

const WebSocket = require('ws');

class SimpleGameValidator {
  constructor() {
    this.apiUrl = 'http://localhost:8000/api';
    this.wsUrl = 'ws://localhost:8000/api/lobbies';
  }

  async validateGame(gameId) {
    console.log(`\n🎮 Validating ${gameId}...`);
    
    try {
      // 1. Create lobby
      const response = await fetch(`${this.apiUrl}/lobbies`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: gameId })
      });
      
      if (!response.ok) {
        throw new Error(`Failed to create lobby: ${response.status}`);
      }
      
      const lobby = await response.json();
      console.log(`  ✓ Lobby created: ${lobby.id}`);
      
      // 2. Connect two players
      const player1 = await this.connectPlayer(lobby.id, 'Player1');
      const player2 = await this.connectPlayer(lobby.id, 'Player2');
      
      console.log(`  ✓ Players connected`);
      
      // 3. Start game
      await this.startGame(player1, lobby.id, gameId);
      console.log(`  ✓ Game started`);
      
      // 4. Make one test move to verify action processing
      await this.makeTestMove(player1, lobby.id, gameId);
      console.log(`  ✓ Test move completed`);
      
      // 5. Cleanup
      player1.close();
      player2.close();
      
      await this.deleteLobby(lobby.id);
      console.log(`  ✓ Cleanup completed`);
      
      return true;
      
    } catch (error) {
      console.log(`  ❌ Failed: ${error.message}`);
      return false;
    }
  }

  async connectPlayer(lobbyId, playerName) {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(`${this.wsUrl}/${lobbyId}/ws?player=${playerName}&join=true`);
      
      ws.on('open', () => resolve(ws));
      ws.on('error', reject);
      
      // Timeout after 5 seconds
      setTimeout(() => reject(new Error('Connection timeout')), 5000);
    });
  }

  async startGame(playerWs, lobbyId, gameId) {
    return new Promise((resolve, reject) => {
      let gameStarted = false;
      let lobbyState = null;
      
      playerWs.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        
        if (msg.type === 'lobbyState') {
          lobbyState = msg.lobby;
        }
        
        if (msg.type === 'gameStarted' && !gameStarted) {
          gameStarted = true;
          resolve();
        }
      });
      
      // Step 1: Create game
      playerWs.send(JSON.stringify({ 
        action: 'createGame', 
        gameType: gameId
      }));
      
      // Step 2: Wait and join game
      setTimeout(() => {
        if (lobbyState?.games?.[0]) {
          const game = lobbyState.games[0];
          
          // Join the game
          playerWs.send(JSON.stringify({ 
            action: 'joinGame', 
            gameId: game.id 
          }));
          
          // Start the game
          setTimeout(() => {
            playerWs.send(JSON.stringify({ 
              action: 'startGame', 
              gameId: game.id 
            }));
          }, 500);
        } else {
          reject(new Error('No game created in lobby'));
        }
      }, 1000);
      
      // Timeout after 15 seconds
      setTimeout(() => {
        if (!gameStarted) {
          reject(new Error('Game start timeout'));
        }
      }, 15000);
    });
  }

  async makeTestMove(playerWs, lobbyId, gameId) {
    return new Promise((resolve, reject) => {
      let moveCompleted = false;
      
      playerWs.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'gameUpdate' && !moveCompleted) {
          moveCompleted = true;
          resolve();
        }
      });
      
      // Send a basic move based on game type
      const testMoves = {
        'tic-tac-toe': {
          action: 'placeMark',
          location: '/zones/board/cells/0/0',
          entity: 'mark_p1'
        },
        'connect-four': {
          action: 'dropChecker',
          targetColumn: 0,
          player: 'p1'
        },
        'three-mens-morris': {
          action: 'placePiece',
          location: '/zones/board/cells/0/0',
          entity: 'piece_p1'
        }
      };
      
      const move = testMoves[gameId] || testMoves['tic-tac-toe'];
      playerWs.send(JSON.stringify({ action: 'gameAction', data: move }));
      
      // Timeout after 5 seconds
      setTimeout(() => {
        if (!moveCompleted) {
          reject(new Error('Move timeout'));
        }
      }, 5000);
    });
  }

  async deleteLobby(lobbyId) {
    try {
      await fetch(`${this.apiUrl}/lobbies/${lobbyId}`, { method: 'DELETE' });
    } catch (error) {
      // Ignore cleanup errors
    }
  }

  async wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

async function validateAllGames() {
  const validator = new SimpleGameValidator();
  
  // List of games to validate (from games/ directory)
  const games = [
    'tic-tac-toe',
    'connect-four', 
    'three-mens-morris',
    'go-fish',
    'war',
    'crazy-eights',
    'old-maid',
    'hex-tic-tac-toe',
    'hex-territory',
    'age-of-steam-mini'
  ];
  
  console.log('🔍 Simple Game Validation Suite');
  console.log('================================');
  
  let passCount = 0;
  let failCount = 0;
  
  for (const game of games) {
    const success = await validator.validateGame(game);
    if (success) {
      passCount++;
    } else {
      failCount++;
    }
    
    // Wait between tests to avoid overwhelming the server
    await validator.wait(1000);
  }
  
  console.log('\n📊 Results:');
  console.log(`✅ Passed: ${passCount}`);
  console.log(`❌ Failed: ${failCount}`);
  console.log(`🎯 Total: ${passCount + failCount}`);
  
  process.exit(failCount > 0 ? 1 : 0);
}

// Run validation if this script is executed directly
if (require.main === module) {
  validateAllGames().catch(error => {
    console.error('Validation failed:', error);
    process.exit(1);
  });
}

module.exports = { SimpleGameValidator };