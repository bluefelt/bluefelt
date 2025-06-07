const WebSocket = require('ws');

/**
 * Test to verify Go Fish deck shuffling
 * This test runs multiple games and verifies that cards are dealt in different orders
 */

const SERVER_URL = 'ws://localhost:8000';
const API_URL = 'http://localhost:8000';

// Track dealt cards across multiple games
const gameDeals = [];

async function createLobby() {
  const response = await fetch(`${API_URL}/api/lobbies`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'go-fish' })
  });
  const data = await response.json();
  return data.id;
}

function connectPlayer(lobbyId, playerName) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=${playerName}&join=true`);
    const dealtCards = [];
    let gameStarted = false;
    
    ws.on('open', () => {
      console.log(`${playerName} connected`);
    });
    
    ws.on('message', (data) => {
      const msg = JSON.parse(data);
      
      // Track when game actually starts
      if (msg.patches) {
        for (const patch of msg.patches) {
          // Look for cards being added to this player's hand
          if (patch.path.match(/\/game\/zones\/hand_p[12]\/items\/\d+/) && patch.op === 'add') {
            if (patch.value && patch.value.entity) {
              dealtCards.push(patch.value.entity);
            }
          }
          // Track when we move past dealing phase
          if (patch.path === '/game/phases/game' && patch.value === 'selectingRank') {
            gameStarted = true;
          }
        }
      }
    });
    
    ws.on('error', reject);
    
    // Resolve with both the websocket and the dealt cards tracker
    resolve({ ws, dealtCards, playerName, gameStarted: () => gameStarted });
  });
}

async function wait(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function testSingleGame(gameNum) {
  console.log(`\n🎮 Game ${gameNum}:`);
  
  const lobbyId = await createLobby();
  console.log(`Created lobby: ${lobbyId}`);
  
  // Connect players
  const player1 = await connectPlayer(lobbyId, 'Alice');
  const player2 = await connectPlayer(lobbyId, 'Bob');
  
  await wait(500);
  
  // Start the game
  console.log('Starting game...');
  player1.ws.send(JSON.stringify({ action: 'start_game' }));
  
  // Wait for dealing to complete
  console.log('Waiting for cards to be dealt...');
  let attempts = 0;
  while (!player1.gameStarted() && attempts < 20) {
    await wait(500);
    attempts++;
  }
  
  // Record the dealt cards for each player
  const gameResult = {
    game: gameNum,
    player1Cards: [...player1.dealtCards].sort(),
    player2Cards: [...player2.dealtCards].sort()
  };
  
  console.log(`Player 1 received: ${gameResult.player1Cards.join(', ')}`);
  console.log(`Player 2 received: ${gameResult.player2Cards.join(', ')}`);
  
  // Clean up
  player1.ws.close();
  player2.ws.close();
  
  return gameResult;
}

async function runShuffleTest() {
  console.log('🃏 Go Fish Deck Shuffling Test\n');
  console.log('This test verifies that the deck is properly shuffled by running multiple games');
  console.log('and checking that players receive different cards each time.\n');
  
  const numGames = 5;
  
  // Run multiple games
  for (let i = 1; i <= numGames; i++) {
    const result = await testSingleGame(i);
    gameDeals.push(result);
    await wait(1000); // Wait between games
  }
  
  // Analyze results
  console.log('\n📊 Analysis:\n');
  
  // Check if any two games had identical deals
  let identicalDeals = 0;
  for (let i = 0; i < gameDeals.length; i++) {
    for (let j = i + 1; j < gameDeals.length; j++) {
      const game1 = gameDeals[i];
      const game2 = gameDeals[j];
      
      // Compare player 1's cards
      const p1Same = game1.player1Cards.length === game2.player1Cards.length &&
                     game1.player1Cards.every((card, idx) => card === game2.player1Cards[idx]);
      
      // Compare player 2's cards  
      const p2Same = game1.player2Cards.length === game2.player2Cards.length &&
                     game2.player2Cards.every((card, idx) => card === game2.player2Cards[idx]);
                     
      if (p1Same && p2Same) {
        console.log(`❌ Games ${i+1} and ${j+1} had IDENTICAL deals!`);
        identicalDeals++;
      }
    }
  }
  
  // Count unique first cards dealt
  const firstCardsP1 = gameDeals.map(g => g.player1Cards[0]).filter(Boolean);
  const uniqueFirstCards = new Set(firstCardsP1).size;
  
  console.log(`\nFirst card dealt to Player 1:`);
  firstCardsP1.forEach((card, i) => {
    console.log(`  Game ${i+1}: ${card}`);
  });
  
  console.log(`\n📈 Statistics:`);
  console.log(`  Games played: ${numGames}`);
  console.log(`  Identical deals found: ${identicalDeals}`);
  console.log(`  Unique first cards to P1: ${uniqueFirstCards} out of ${firstCardsP1.length}`);
  
  // Verdict
  console.log('\n🏁 Result:');
  if (identicalDeals === 0 && uniqueFirstCards > 1) {
    console.log('✅ PASS: Deck appears to be properly shuffled!');
    console.log('   Different cards were dealt in each game.');
  } else if (identicalDeals > 0) {
    console.log('❌ FAIL: Found identical deals across games!');
    console.log('   The deck is NOT being shuffled properly.');
  } else if (uniqueFirstCards === 1) {
    console.log('❌ FAIL: Same first card dealt every game!');
    console.log('   The deck is NOT being shuffled properly.');
  }
  
  // Show detailed comparison of first two games
  if (gameDeals.length >= 2) {
    console.log('\n🔍 Detailed comparison of first two games:');
    console.log('\nGame 1 vs Game 2 - Player 1 cards:');
    for (let i = 0; i < 7; i++) {
      const card1 = gameDeals[0].player1Cards[i] || 'none';
      const card2 = gameDeals[1].player1Cards[i] || 'none';
      const same = card1 === card2 ? '❌ SAME' : '✅ different';
      console.log(`  Position ${i+1}: ${card1} vs ${card2} - ${same}`);
    }
  }
}

// Run the test
runShuffleTest().catch(console.error);