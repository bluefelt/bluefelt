const WebSocket = require('ws');

async function testGoFishLogging() {
    console.log('Testing Go Fish automatic action logging...\n');
    
    // Create a lobby for Go Fish
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'go-fish' })
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
                    console.log(`LOG: [${patch.value.timestamp}] ${patch.value.message}`);
                }
            });
        }
    });
    
    await new Promise(resolve => {
        bob.on('open', resolve);
    });
    
    // Start the game - this should trigger automatic dealing actions
    await new Promise(resolve => setTimeout(resolve, 100));
    console.log('\nStarting game...\n');
    alice.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for automatic actions to process
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Print summary
    console.log('\n\nSummary:');
    console.log('=========');
    console.log(`Total logs: ${logs.length}`);
    
    // Check for automatic action logs
    const automaticLogs = logs.filter(log => log.auto === true);
    console.log(`Automatic action logs: ${automaticLogs.length}`);
    
    console.log('\nChecking for specific automatic actions:');
    const hasDealingLogs = logs.some(log => log.message.includes('Dealing'));
    const hasGameStart = logs.some(log => log.message.includes('Starting game'));
    
    console.log(`✓ Dealing cards logged: ${hasDealingLogs ? 'YES' : 'NO'}`);
    console.log(`✓ Game start logged: ${hasGameStart ? 'YES' : 'NO'}`);
    
    alice.close();
    bob.close();
}

testGoFishLogging().catch(console.error);