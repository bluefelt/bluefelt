const WebSocket = require('ws');

async function testAutomaticLogging() {
    console.log('Testing automatic action logging...\n');
    
    // Create a lobby for Tic-Tac-Toe
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect two players
    const alice = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=alice&join=true`);
    const bob = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=bob&join=true`);
    
    const logs = [];
    
    // Collect all log messages
    alice.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'diff' && msg.patch) {
            msg.patch.forEach(patch => {
                if (patch.path && patch.path.includes('/ui/gameLog') && patch.op === 'add') {
                    logs.push(patch.value);
                }
            });
        }
    });
    
    await new Promise(resolve => {
        bob.on('open', resolve);
    });
    
    // Start the game
    await new Promise(resolve => setTimeout(resolve, 100));
    alice.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for game to start
    await new Promise(resolve => setTimeout(resolve, 200));
    
    // Make a move
    alice.send(JSON.stringify({
        action: 'placeMarker',
        args: {
            location: '/zones/board/cells/1/1',
            entity: 'mark_p1'
        }
    }));
    
    // Wait for automatic actions to process
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Print all logs
    console.log('\nGame Logs:');
    console.log('==========');
    logs.forEach((log, i) => {
        console.log(`${i + 1}. [${log.timestamp}] ${log.message}`);
    });
    
    // Check for specific automatic action logs
    console.log('\nChecking for automatic action logs:');
    const hasWinCheck = logs.some(log => log.message.includes('Checking for three in a row'));
    const hasTurnAdvance = logs.some(log => log.message.includes('Turn passes to'));
    
    console.log(`✓ Win condition check logged: ${hasWinCheck ? 'YES' : 'NO'}`);
    console.log(`✓ Turn advancement logged: ${hasTurnAdvance ? 'YES' : 'NO'}`);
    
    alice.close();
    bob.close();
}

testAutomaticLogging().catch(console.error);