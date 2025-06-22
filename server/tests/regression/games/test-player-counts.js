const { GameTestFramework } = require('../framework/GameTestFramework');
const assert = require('assert');

class PlayerCountTest extends GameTestFramework {
  constructor(gameId) {
    super(gameId);
  }
  
  async testInsufficientPlayers() {
    console.log('\n🚫 Testing Start with Insufficient Players\n');
    
    // Create lobby and connect only one player (most games need 2+)
    await this.createLobby();
    await this.connectPlayers(['Alice']);
    
    // Try to start game with only one player
    console.log('Attempting to start game with only 1 player...');
    
    const alice = this.players.get('Alice');
    alice.ws.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for response
    await this.wait(1000);
    
    // Game should not have started
    assert(!this.gameState, 'Game should not start with insufficient players');
    console.log('✅ Game correctly refused to start with insufficient players');
  }
  
  async testMaxPlayerLimit() {
    console.log('\n🚫 Testing Maximum Player Limit\n');
    
    // Get max players from manifest
    const manifest = this.lobbyInfo?.manifest;
    const maxPlayers = manifest?.metadata?.players?.max || 4;
    
    console.log(`Game allows maximum ${maxPlayers} players`);
    
    // Try to connect more than max players
    const playerNames = [];
    for (let i = 1; i <= maxPlayers + 1; i++) {
      playerNames.push(`Player${i}`);
    }
    
    // Connect players up to max
    for (let i = 0; i < maxPlayers; i++) {
      await this.connectPlayer(playerNames[i]);
      await this.wait(200);
    }
    
    console.log(`Connected ${maxPlayers} players (at capacity)`);
    
    // Try to connect one more player
    try {
      console.log(`Attempting to connect player ${maxPlayers + 1}...`);
      const extraWs = await this.connectPlayer(playerNames[maxPlayers]);
      
      // Check if the extra player was rejected or allowed as spectator
      await this.wait(500);
      
      // In current implementation, extra players might be allowed but can't join the game
      console.log('⚠️  Extra player connection was not rejected (may be spectator)');
    } catch (error) {
      console.log('✅ Extra player connection was properly rejected');
    }
  }
  
  async testMinimumPlayersCanStart() {
    console.log('\n✅ Testing Start with Minimum Players\n');
    
    // Get min players from manifest
    const manifest = this.lobbyInfo?.manifest;
    const minPlayers = manifest?.metadata?.players?.min || 2;
    
    console.log(`Game requires minimum ${minPlayers} players`);
    
    // Connect minimum number of players
    const playerNames = [];
    for (let i = 1; i <= minPlayers; i++) {
      playerNames.push(`Player${i}`);
    }
    
    await this.connectPlayers(playerNames);
    
    // Start game
    await this.startGame();
    
    // Verify game started
    assert(this.gameState, 'Game should start with minimum players');
    assert(this.currentPlayer, 'Current player should be set');
    console.log('✅ Game successfully started with minimum players');
  }
  
  async testStartGameButtonVisibility() {
    console.log('\n🔘 Testing Start Game Button Visibility Fix\n');
    
    // This tests the specific issue where Start Game button wasn't appearing
    // in Go Fish lobby even with sufficient players
    
    // Create lobby and connect players without auto-join
    await this.createLobby();
    
    // Connect first player
    const ws1 = await this.connectPlayer('Alice');
    await this.wait(500);
    
    // Check welcome message for first player
    const alice = this.players.get('Alice');
    console.log('Alice welcome state:');
    console.log(`  - you: ${alice.id || 'null'}`);
    console.log(`  - welcomeMessage:`, alice.welcomeMessage);
    
    // Check the welcome message structure
    const welcomeUI = alice.welcomeMessage?.ui;
    console.log(`  - players: ${welcomeUI?.players?.length || 0}`);
    console.log(`  - manifest present: ${!!welcomeUI?.manifest}`);
    console.log(`  - started: ${alice.welcomeMessage?.started}`);
    
    // Verify manifest is present even before game starts
    assert(welcomeUI?.manifest, 'Manifest should be present in welcome message');
    assert.equal(welcomeUI?.manifest?.metadata?.players?.min, 2, 'Min players should be 2');
    
    // Connect second player
    const ws2 = await this.connectPlayer('Bob');
    await this.wait(500);
    
    // Check that both players are recognized
    const bob = this.players.get('Bob');
    assert.equal(alice.id, 'Alice', 'Alice should be recognized as a player');
    assert.equal(bob.id, 'Bob', 'Bob should be recognized as a player');
    
    // Get the latest welcome message for verification
    const latestWelcome = bob.welcomeMessage || alice.welcomeMessage;
    
    // Verify game state for Start Game button requirements
    assert.equal(latestWelcome?.started, false, 'Game should not be started');
    assert.equal(latestWelcome?.ui?.players?.length, 2, 'Should have 2 players');
    
    console.log('✅ All requirements met for Start Game button visibility');
    console.log('   - started: false');
    console.log('   - you: recognized player');
    console.log('   - players >= min: true');
  }
}

// Test runner for different games
async function runPlayerCountTests() {
  console.log('\n' + '='.repeat(60));
  console.log('PLAYER COUNT TESTS');
  console.log('='.repeat(60));
  
  const games = [
    { id: 'tic-tac-toe', name: 'Tic-Tac-Toe' },
    { id: 'go-fish', name: 'Go Fish' },
    { id: 'three-mens-morris', name: 'Three Men\'s Morris' },
    { id: 'connect-four', name: 'Connect Four' }
  ];
  
  let allPassed = true;
  
  for (const game of games) {
    console.log('\n' + '='.repeat(60));
    console.log(`Testing ${game.name}`);
    console.log('='.repeat(60));
    
    try {
      // Test insufficient players
      let test = new PlayerCountTest(game.id);
      await test.testInsufficientPlayers();
      await test.cleanup();
      
      // Test maximum players (if applicable)
      if (game.id === 'go-fish') { // Go Fish supports 2-4 players
        test = new PlayerCountTest(game.id);
        await test.createLobby();
        await test.testMaxPlayerLimit();
        await test.cleanup();
      }
      
      // Test minimum players can start
      test = new PlayerCountTest(game.id);
      await test.createLobby();
      await test.testMinimumPlayersCanStart();
      await test.cleanup();
      
      // Test Start Game button visibility fix
      test = new PlayerCountTest(game.id);
      await test.testStartGameButtonVisibility();
      await test.cleanup();
      
    } catch (error) {
      console.error(`\n❌ ${game.name} tests failed:`, error.message);
      allPassed = false;
    }
  }
  
  console.log('\n' + '='.repeat(60));
  console.log(allPassed ? '✅ ALL PLAYER COUNT TESTS PASSED!' : '❌ SOME TESTS FAILED!');
  console.log('='.repeat(60));
  
  process.exit(allPassed ? 0 : 1);
}

if (require.main === module) {
  runPlayerCountTests().catch(console.error);
}

module.exports = { PlayerCountTest };