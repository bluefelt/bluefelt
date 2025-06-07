const WebSocket = require('ws');

async function simpleLogTest() {
    console.log('Simple log test for Tic Tac Toe...\n');
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect player
    const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    
    const allMessages = [];
    const logs = [];
    
    ws.on('message', (data) => {
        const msg = JSON.parse(data);
        allMessages.push(msg);
        
        console.log(`\nReceived ${msg.type} message`);
        
        if (msg.patches) {
            console.log('Patches:');
            msg.patches.forEach(patch => {
                console.log(`  - ${patch.op} ${patch.path}`);
                if (patch.path && patch.path.includes('gameLog')) {
                    console.log('\n📝 GAME LOG FOUND:');
                    console.log('Message:', patch.value.message);
                    console.log('Full value:', JSON.stringify(patch.value, null, 2));
                    logs.push(patch.value);
                    
                    // Check for unresolved variables
                    if (patch.value.message) {
                        const unresolved = patch.value.message.match(/\{[^}]+\}/g);
                        if (unresolved) {
                            console.log('⚠️  Unresolved variables:', unresolved);
                        }
                    }
                }
            });
        }
    });
    
    // Wait for connection
    await new Promise(resolve => ws.on('open', resolve));
    
    // Add second player
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    await new Promise(resolve => ws2.on('open', resolve));
    
    // Start game
    console.log('Starting game...');
    ws.send(JSON.stringify({ action: 'start_game' }));
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Make a move
    console.log('\nMaking move at 1,1...');
    ws.send(JSON.stringify({
        action: 'placeMarker',
        location: '/zones/board/cells/1/1'
    }));
    
    // Wait for response
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Summary
    console.log('\n=== SUMMARY ===');
    console.log('Total messages:', allMessages.length);
    console.log('Total logs:', logs.length);
    
    if (logs.length === 0) {
        console.log('\n⚠️  No logs captured. Checking all messages...');
        allMessages.forEach((msg, i) => {
            console.log(`\nMessage ${i + 1}:`, msg.type);
            if (msg.patches) {
                console.log('Patches:', msg.patches.map(p => `${p.op} ${p.path}`).join(', '));
            }
        });
    }
    
    ws.close();
    ws2.close();
}

simpleLogTest().catch(console.error);