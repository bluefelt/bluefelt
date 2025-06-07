const WebSocket = require('ws');

/**
 * Simple test to verify Go Fish deck shuffling by checking the pool order after dealing
 */

const SERVER_URL = 'ws://localhost:8000';
const API_URL = 'http://localhost:8000';

async function createLobby() {
  const response = await fetch(`${API_URL}/api/lobbies`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'go-fish' })
  });
  const data = await response.json();
  return data.id;
}

function wait(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function connectAndCheckDealing(lobbyId, playerName) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=${playerName}&join=true`);
    const dealtCards = [];
    let gameStarted = false;
    let poolAfterDealing = null;
    
    ws.on('open', () => {
      console.log(`${playerName} connected`);
    });
    
    ws.on('message', (data) => {
      const msg = JSON.parse(data);
      
      // Look for patches indicating cards dealt
      if (msg.patches) {
        for (const patch of msg.patches) {
          // Track cards added to any player's hand
          if (patch.path.includes('/zones/hand_') && patch.op === 'add' && patch.value?.entity) {
            dealtCards.push(patch.value.entity);
          }
          
          // When phase changes to selectingRank, dealing is done
          if (patch.path === '/game/phases/game' && patch.value === 'selectingRank') {
            gameStarted = true;
          }
        }
      }
      
      // In lobbyState messages, check the pool after dealing
      if (msg.lobbyState?.zones?.pool && gameStarted) {
        poolAfterDealing = msg.lobbyState.zones.pool.items.map(item => item.entity);
      }
    });
    
    ws.on('error', reject);
    
    // Check periodically if we're done
    const checkDone = setInterval(() => {
      if (gameStarted && dealtCards.length >= 7) {
        clearInterval(checkDone);
        ws.close();
        resolve({ dealtCards, poolAfterDealing });
      }
    }, 100);
    
    // Timeout after 10 seconds
    setTimeout(() => {
      clearInterval(checkDone);
      ws.close();
      reject(new Error(`Timeout waiting for dealing to complete. Got ${dealtCards.length} cards.`));
    }, 10000);
  });
}

async function testSingleGame(gameNum) {
  console.log(`\n🎮 Game ${gameNum}:`);
  
  const lobbyId = await createLobby();
  console.log(`Created lobby: ${lobbyId}`);
  
  // Connect both players
  const player1Promise = connectAndCheckDealing(lobbyId, 'Alice');
  const player2Promise = connectAndCheckDealing(lobbyId, 'Bob');
  
  await wait(1000);
  
  // Start the game
  console.log('Starting game...');
  const tempWs = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=starter&join=false`);
  tempWs.on('open', () => {
    tempWs.send(JSON.stringify({ action: 'start_game' }));
    tempWs.close();
  });
  
  // Wait for both players to finish
  const [result1, result2] = await Promise.all([player1Promise, player2Promise]);
  
  console.log(`Alice dealt: ${result1.dealtCards.slice(0, 3).join(', ')}... (${result1.dealtCards.length} total)`);
  console.log(`Bob dealt: ${result2.dealtCards.slice(0, 3).join(', ')}... (${result2.dealtCards.length} total)`);
  
  // Combine all dealt cards
  const allDealtCards = [...result1.dealtCards, ...result2.dealtCards];
  
  return {
    game: gameNum,
    allDealtCards: allDealtCards.sort(), // Sort for comparison
    firstCard: allDealtCards[0],
    poolAfterDealing: result1.poolAfterDealing || result2.poolAfterDealing
  };
}

async function runShuffleTest() {
  console.log('🃏 Go Fish Deck Shuffling Test');
  console.log('Running multiple games to verify deck shuffling...\n');
  
  const results = [];
  
  // Run 3 games
  for (let i = 1; i <= 3; i++) {
    try {
      const result = await testSingleGame(i);
      results.push(result);
      await wait(1000); // Wait between games
    } catch (error) {
      console.error(`Game ${i} failed:`, error.message);
    }
  }
  
  // Analyze results
  console.log('\n📊 Analysis:');
  
  if (results.length < 2) {
    console.log('❌ Not enough successful games to analyze shuffling');
    return;
  }
  
  // Check if first cards are different
  const firstCards = results.map(r => r.firstCard);
  const uniqueFirstCards = new Set(firstCards).size;
  
  console.log('\nFirst card dealt in each game:');
  firstCards.forEach((card, i) => {
    console.log(`  Game ${i + 1}: ${card}`);
  });
  
  // Check if any games had identical deals
  let identicalDeals = 0;
  for (let i = 0; i < results.length; i++) {
    for (let j = i + 1; j < results.length; j++) {
      const cards1 = results[i].allDealtCards;
      const cards2 = results[j].allDealtCards;
      
      if (cards1.length === cards2.length && 
          cards1.every((card, idx) => card === cards2[idx])) {
        console.log(`⚠️  Games ${i + 1} and ${j + 1} had identical deals!`);
        identicalDeals++;
      }
    }
  }
  
  // Verdict
  console.log('\n🏁 Result:');
  if (identicalDeals === 0 && uniqueFirstCards > 1) {
    console.log('✅ PASS: Deck appears to be properly shuffled!');
    console.log(`   ${uniqueFirstCards} different first cards across ${results.length} games.`);
  } else if (identicalDeals > 0) {
    console.log('❌ FAIL: Found identical deals across games!');
    console.log('   The deck is NOT being shuffled properly.');
  } else if (uniqueFirstCards === 1) {
    console.log('⚠️  SUSPICIOUS: Same first card in all games');
    console.log('   This could indicate shuffling issues or just bad luck.');
  }
}

runShuffleTest().catch(console.error);