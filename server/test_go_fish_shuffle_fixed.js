const WebSocket = require('ws');

async function testGoFishShuffle() {
    const results = [];
    
    for (let i = 0; i < 3; i++) {
        console.log(`\n=== Game ${i + 1} ===`);
        
        // Create new lobby
        const resp = await fetch('http://localhost:8000/api/lobbies', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ game_id: 'go-fish' })
        });
        const newLobby = await resp.json();
        console.log('Created lobby:', newLobby.id);
        
        // Connect two players
        const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${newLobby.id}/ws?player=Player1&join=true`);
        const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${newLobby.id}/ws?player=Player2&join=true`);
        
        let gameState = null;
        let gotState = false;
        
        // Wait for both connections
        await new Promise(resolve => {
            let connected = 0;
            ws1.on('open', () => { connected++; if (connected === 2) resolve(); });
            ws2.on('open', () => { connected++; if (connected === 2) resolve(); });
        });
        
        // Listen for state updates from both websockets
        const statePromise = new Promise(resolve => {
            const checkState = (data) => {
                try {
                    const msg = JSON.parse(data);
                    if (msg.type === 'state' && msg.data && msg.data.game && msg.data.game.zones) {
                        gameState = msg.data;
                        // Check if cards have been dealt
                        if (gameState.game.zones.hand_p1 && gameState.game.zones.hand_p1.length > 0) {
                            gotState = true;
                            resolve();
                        }
                    }
                } catch (e) {
                    // Ignore parse errors
                }
            };
            
            ws1.on('message', checkState);
            ws2.on('message', checkState);
        });
        
        // Start game
        ws1.send(JSON.stringify({ action: 'start_game' }));
        
        // Wait for state update with timeout
        await Promise.race([
            statePromise,
            new Promise(resolve => setTimeout(resolve, 5000))
        ]);
        
        if (gotState && gameState && gameState.game && gameState.game.zones) {
            const p1Hand = gameState.game.zones.hand_p1 || [];
            const p2Hand = gameState.game.zones.hand_p2 || [];
            
            console.log('Player 1 cards:', p1Hand.map(c => `${c.rank}${c.suit}`).join(', '));
            console.log('Player 2 cards:', p2Hand.map(c => `${c.rank}${c.suit}`).join(', '));
            
            results.push({
                p1: p1Hand.map(c => `${c.rank}${c.suit}`),
                p2: p2Hand.map(c => `${c.rank}${c.suit}`)
            });
        } else {
            console.log('ERROR: No game state received');
        }
        
        ws1.close();
        ws2.close();
        
        // Small delay between games
        await new Promise(resolve => setTimeout(resolve, 500));
    }
    
    // Check if cards are different across games
    console.log('\n=== Checking for shuffle ===');
    
    if (results.length < 2) {
        console.log('❌ ERROR: Not enough games completed');
        return;
    }
    
    // Compare first game to others
    let allSame = true;
    for (let i = 1; i < results.length; i++) {
        const game1Cards = [...results[0].p1, ...results[0].p2].join(',');
        const gameNCards = [...results[i].p1, ...results[i].p2].join(',');
        
        if (game1Cards !== gameNCards) {
            allSame = false;
            break;
        }
    }
    
    if (allSame) {
        console.log('❌ DECK NOT SHUFFLED - Same cards dealt in all games!');
        console.log('Game 1 P1:', results[0].p1.slice(0, 5).join(', '));
        console.log('Game 2 P1:', results[1].p1.slice(0, 5).join(', '));
        console.log('Game 3 P1:', results[2].p1.slice(0, 5).join(', '));
    } else {
        console.log('✅ DECK IS SHUFFLED - Different cards dealt across games');
    }
}

testGoFishShuffle().catch(console.error);