const WebSocket = require('ws');

async function testConnect4Logging() {
    console.log('Testing Connect 4 logging...\n');
    
    // Create a lobby for Connect 4
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'connect-four' })
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
    
    // Make a move with targetColumn
    alice.send(JSON.stringify({
        action: 'dropDisc',
        args: {
            zone: '/zones/board',
            targetColumn: 3,
            entity: 'disc_p1'
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
    
    // Check for variable replacement
    console.log('\nChecking for proper variable replacement:');
    const hasTargetColumn = logs.some(log => log.message.includes('column 4')); // 1-indexed
    const hasBrokenVariable = logs.some(log => log.message.includes('{') && log.message.includes('}'));
    
    console.log(`✓ Column number correctly replaced: ${hasTargetColumn ? 'YES' : 'NO'}`);
    console.log(`✓ No broken variables: ${!hasBrokenVariable ? 'YES' : 'NO'}`);
    
    alice.close();
    bob.close();
}

testConnect4Logging().catch(console.error);