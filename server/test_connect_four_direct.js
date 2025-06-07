const WebSocket = require('ws');

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function testConnectFour() {
    console.log('Direct Connect Four test...\n');
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'connect-four' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect first player
    const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=TestPlayer&join=true`);
    
    let messageCount = 0;
    
    ws.on('message', (data) => {
        messageCount++;
        const msg = JSON.parse(data);
        console.log(`\nMessage ${messageCount}: ${msg.type}`);
        
        if (msg.type === 'welcome') {
            console.log('State structure:', JSON.stringify(msg.state, null, 2));
        }
        
        if (msg.type === 'diff' && msg.patch) {
            console.log('Patches:');
            for (const patch of msg.patch) {
                console.log(`  - ${patch.op} ${patch.path}`);
                if (patch.value !== undefined) {
                    console.log(`    Value: ${JSON.stringify(patch.value).substring(0, 100)}...`);
                }
            }
        }
        
        if (msg.type === 'error') {
            console.log('ERROR:', msg);
        }
    });
    
    await sleep(1000);
    
    // Start single player game
    console.log('\nStarting game...');
    ws.send(JSON.stringify({ action: 'start_game' }));
    
    await sleep(1000);
    
    // Try to drop a disc
    console.log('\nAttempting to drop disc at column 3...');
    ws.send(JSON.stringify({ 
        action: 'dropDisc',
        targetColumn: 3 
    }));
    
    await sleep(1000);
    
    ws.close();
    console.log('\nTest completed.');
}

testConnectFour().catch(console.error);