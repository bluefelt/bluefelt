const WebSocket = require('ws');

/**
 * Final test to verify shuffling by examining the state after dealing
 */

const SERVER_URL = 'ws://localhost:8000';
const API_URL = 'http://localhost:8000';

// Store the unshuffled deck order for comparison
const UNSHUFFLED_DECK = [
  'card_hearts_a', 'card_hearts_2', 'card_hearts_3', 'card_hearts_4', 'card_hearts_5', 'card_hearts_6', 'card_hearts_7', 
  'card_hearts_8', 'card_hearts_9', 'card_hearts_10', 'card_hearts_j', 'card_hearts_q', 'card_hearts_k',
  'card_diamonds_a', 'card_diamonds_2', 'card_diamonds_3', 'card_diamonds_4', 'card_diamonds_5', 'card_diamonds_6', 'card_diamonds_7',
  'card_diamonds_8', 'card_diamonds_9', 'card_diamonds_10', 'card_diamonds_j', 'card_diamonds_q', 'card_diamonds_k',
  'card_clubs_a', 'card_clubs_2', 'card_clubs_3', 'card_clubs_4', 'card_clubs_5', 'card_clubs_6', 'card_clubs_7',
  'card_clubs_8', 'card_clubs_9', 'card_clubs_10', 'card_clubs_j', 'card_clubs_q', 'card_clubs_k',
  'card_spades_a', 'card_spades_2', 'card_spades_3', 'card_spades_4', 'card_spades_5', 'card_spades_6', 'card_spades_7',
  'card_spades_8', 'card_spades_9', 'card_spades_10', 'card_spades_j', 'card_spades_q', 'card_spades_k'
];

async function createLobby() {
  const response = await fetch(`${API_URL}/api/lobbies`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'go-fish' })
  });
  const data = await response.json();
  return data.id;
}

function examineGameState(lobbyId) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=examiner&join=false`);
    let finalState = null;
    let gameStarted = false;
    
    ws.on('open', () => {
      console.log('Examiner connected');
    });
    
    ws.on('message', (data) => {
      const msg = JSON.parse(data);
      
      // Look for game started indication
      if (msg.type === 'gameStarted') {
        gameStarted = true;
        console.log('Game has started, examining state...');
      }
      
      // Capture any state that includes zones
      if (msg.lobbyState?.zones) {
        finalState = msg.lobbyState;
      }
      
      // If we have state and game has started, we can analyze
      if (gameStarted && finalState) {
        setTimeout(() => {
          ws.close();
          resolve(finalState);
        }, 2000); // Wait a bit for dealing to complete
      }
    });
    
    ws.on('error', reject);
    
    setTimeout(() => {
      ws.close();
      reject(new Error('Timeout waiting for game state'));
    }, 10000);
  });
}

async function testGame(gameNum) {
  console.log(`\n🎮 Game ${gameNum}:`);
  
  const lobbyId = await createLobby();
  console.log(`Created lobby: ${lobbyId}`);
  
  // Start examining the game state
  const statePromise = examineGameState(lobbyId);
  
  // Connect players and start game
  setTimeout(() => {
    const player1 = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=Alice&join=true`);
    player1.on('open', () => {
      setTimeout(() => {
        const player2 = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=Bob&join=true`);
        player2.on('open', () => {
          setTimeout(() => {
            // Start the game
            player1.send(JSON.stringify({ action: 'start_game' }));
            setTimeout(() => {
              player1.close();
              player2.close();
            }, 3000);
          }, 500);
        });
      }, 500);
    });
  }, 500);
  
  const state = await statePromise;
  
  // Analyze the state
  const poolZone = state.zones?.find(z => z.id === 'pool');
  const hand1Zone = state.zones?.find(z => z.id === 'hand_p1');
  const hand2Zone = state.zones?.find(z => z.id === 'hand_p2');
  
  const poolCards = poolZone?.contents?.map(c => c.entity) || [];
  const hand1Cards = hand1Zone?.contents?.map(c => c.entity) || [];
  const hand2Cards = hand2Zone?.contents?.map(c => c.entity) || [];
  
  console.log(`Pool has ${poolCards.length} cards`);
  console.log(`Hand 1 has ${hand1Cards.length} cards`);
  console.log(`Hand 2 has ${hand2Cards.length} cards`);
  
  if (poolCards.length > 0) {
    console.log(`First 5 pool cards: ${poolCards.slice(0, 5).join(', ')}`);
  }
  
  if (hand1Cards.length > 0) {
    console.log(`First 3 hand1 cards: ${hand1Cards.slice(0, 3).join(', ')}`);
  }
  
  // Check if the pool order matches unshuffled deck
  const isUnshuffled = poolCards.length > 10 && 
    poolCards.slice(0, 10).every((card, i) => card === UNSHUFFLED_DECK[i + 14]); // +14 because first 14 cards should be dealt
  
  return {
    game: gameNum,
    poolSize: poolCards.length,
    hand1Size: hand1Cards.length,
    hand2Size: hand2Cards.length,
    firstPoolCard: poolCards[0],
    firstHand1Card: hand1Cards[0],
    isUnshuffled
  };
}

async function runFinalShuffleTest() {
  console.log('🃏 Final Go Fish Shuffling Test');
  console.log('Examining game state after dealing to detect shuffling...\n');
  
  const results = [];
  
  for (let i = 1; i <= 3; i++) {
    try {
      const result = await testGame(i);
      results.push(result);
      console.log(`Game ${i}: Pool=${result.poolSize}, Hands=${result.hand1Size}+${result.hand2Size}, Shuffled=${!result.isUnshuffled}`);
    } catch (error) {
      console.error(`Game ${i} failed:`, error.message);
    }
  }
  
  console.log('\n📊 Analysis:');
  
  if (results.length === 0) {
    console.log('❌ No successful games to analyze');
    return;
  }
  
  // Check shuffling
  const unshuffledGames = results.filter(r => r.isUnshuffled).length;
  const validGames = results.filter(r => r.hand1Size === 7 && r.hand2Size === 7 && r.poolSize === 38).length;
  
  console.log(`\nValid games (7+7 cards dealt, 38 in pool): ${validGames}/${results.length}`);
  console.log(`Games with unshuffled deck order: ${unshuffledGames}/${results.length}`);
  
  // Check for variety in first cards
  const firstPoolCards = results.map(r => r.firstPoolCard).filter(Boolean);
  const firstHand1Cards = results.map(r => r.firstHand1Card).filter(Boolean);
  const uniquePoolCards = new Set(firstPoolCards).size;
  const uniqueHand1Cards = new Set(firstHand1Cards).size;
  
  console.log(`\nFirst pool cards: ${firstPoolCards.join(', ')}`);
  console.log(`First hand1 cards: ${firstHand1Cards.join(', ')}`);
  console.log(`Unique first pool cards: ${uniquePoolCards}/${firstPoolCards.length}`);
  console.log(`Unique first hand1 cards: ${uniqueHand1Cards}/${firstHand1Cards.length}`);
  
  // Final verdict
  console.log('\n🏁 Result:');
  if (validGames === 0) {
    console.log('❌ FAIL: No cards were dealt properly');
  } else if (unshuffledGames === results.length) {
    console.log('❌ FAIL: All games showed unshuffled deck order');
  } else if (unshuffledGames > 0) {
    console.log('⚠️  MIXED: Some games appear shuffled, others not');
  } else if (uniquePoolCards > 1 || uniqueHand1Cards > 1) {
    console.log('✅ PASS: Deck appears to be shuffled (different cards each game)');
  } else {
    console.log('⚠️  INCONCLUSIVE: Too few games or insufficient variation to determine shuffling');
  }
}

runFinalShuffleTest().catch(console.error);