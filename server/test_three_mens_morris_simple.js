const WebSocket = require('ws');

async function testThreeMensMorris() {
    console.log('Testing Three Men\'s Morris...\n');
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'three-mens-morris' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect two players
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    let gameStarted = false;
    let logs = [];
    
    // Collect messages from player 1
    ws1.on('message', (data) => {
        const msg = JSON.parse(data);
        
        if (msg.state) {
            console.log('\n=== INITIAL STATE ===');
            console.log('Current player:', msg.state.currentPlayer);
            console.log('Phase:', JSON.stringify(msg.state.phases));
            console.log('UI:', JSON.stringify(msg.ui, null, 2));
        }
        
        if (msg.error) {
            console.log('\n❌ ERROR:', msg.error);
        }
        
        if (msg.patches) {
            msg.patches.forEach(patch => {
                if (patch.path === '/game/started' && patch.value === true) {
                    gameStarted = true;
                    console.log('\n✅ Game started!');
                }
                
                if (patch.path && patch.path.includes('/ui/gameLog')) {
                    console.log('\n📝 GAME LOG:', patch.value);
                    logs.push(patch.value);
                }
            });
        }
    });
    
    // Wait for connections
    await new Promise(resolve => {
        let connected = 0;
        ws1.on('open', () => { connected++; if (connected === 2) resolve(); });
        ws2.on('open', () => { connected++; if (connected === 2) resolve(); });
    });
    
    console.log('\nBoth players connected');
    
    // Start game
    ws1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for game to start
    await new Promise(resolve => {
        const check = setInterval(() => {
            if (gameStarted) {
                clearInterval(check);
                resolve();
            }
        }, 100);
    });
    
    // Now try placing a piece with the correct format
    console.log('\nAttempting to place piece...');
    
    // The action expects:
    // - action: 'placeToken'
    // - target: position index (0-8)
    // - entity: the piece to place
    
    ws1.send(JSON.stringify({
        action: 'placeToken',
        target: '0',
        entity: 'piece_p1'
    }));
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Summary
    console.log('\n=== SUMMARY ===');
    console.log('Total logs collected:', logs.length);
    if (logs.length > 0) {
        console.log('\nGame logs:');
        logs.forEach((log, i) => {
            console.log(`${i + 1}. [${log.timestamp}] ${log.message}`);
        });
    }
    
    // Close connections
    ws1.close();
    ws2.close();
}

testThreeMensMorris().catch(console.error);