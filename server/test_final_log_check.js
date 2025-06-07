const WebSocket = require('ws');

async function finalLogTest() {
    console.log('Final comprehensive log test...\n');
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Set up both players BEFORE connecting
    const logs = [];
    let messageCount = 0;
    
    const messageHandler = (playerName) => (data) => {
        const msg = JSON.parse(data);
        messageCount++;
        
        if (msg.type === 'diff' && msg.patch) {
            console.log(`\n[${playerName}] Received diff #${msg.tick}`);
            msg.patch.forEach(patch => {
                console.log(`  ${patch.op} ${patch.path}`);
                if (patch.path && patch.path.includes('gameLog')) {
                    console.log(`\n📝 GAME LOG from ${playerName}:`);
                    console.log(`  Message: "${patch.value.message}"`);
                    console.log(`  Timestamp: ${patch.value.timestamp}`);
                    logs.push({
                        player: playerName,
                        ...patch.value
                    });
                    
                    // Check for unresolved variables
                    if (patch.value.message) {
                        const unresolved = patch.value.message.match(/\{[^}]+\}/g);
                        if (unresolved) {
                            console.log(`  ⚠️  UNRESOLVED: ${unresolved.join(', ')}`);
                        }
                    }
                }
            });
        }
    };
    
    // Connect both players
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    ws1.on('message', messageHandler('Alice'));
    ws2.on('message', messageHandler('Bob'));
    
    // Wait for both connections
    await Promise.all([
        new Promise(resolve => ws1.on('open', resolve)),
        new Promise(resolve => ws2.on('open', resolve))
    ]);
    
    console.log('Both players connected');
    
    // Start game
    console.log('\nStarting game...');
    ws1.send(JSON.stringify({ action: 'start_game' }));
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Make moves
    console.log('\nAlice placing mark at center (1,1)...');
    ws1.send(JSON.stringify({
        action: 'placeMarker',
        location: '/zones/board/cells/1/1'
    }));
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    console.log('\nBob placing mark at corner (0,0)...');
    ws2.send(JSON.stringify({
        action: 'placeMarker',
        location: '/zones/board/cells/0/0'
    }));
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    console.log('\nAlice placing mark at (0,1)...');
    ws1.send(JSON.stringify({
        action: 'placeMarker',
        location: '/zones/board/cells/0/1'
    }));
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Summary
    console.log('\n=== FINAL SUMMARY ===');
    console.log(`Total messages received: ${messageCount}`);
    console.log(`Total game logs: ${logs.length}`);
    
    if (logs.length > 0) {
        console.log('\n✅ Game logs captured:');
        logs.forEach((log, i) => {
            console.log(`${i + 1}. [${log.timestamp}] ${log.message} (seen by ${log.player})`);
        });
        
        const unresolvedLogs = logs.filter(log => 
            log.message && log.message.match(/\{[^}]+\}/)
        );
        
        if (unresolvedLogs.length > 0) {
            console.log('\n❌ LOGS WITH UNRESOLVED TEMPLATE VARIABLES:');
            unresolvedLogs.forEach(log => {
                console.log(`  - "${log.message}"`);
                const vars = log.message.match(/\{[^}]+\}/g);
                console.log(`    Variables: ${vars.join(', ')}`);
            });
        } else {
            console.log('\n✅ All template variables resolved correctly!');
        }
    } else {
        console.log('\n❌ No game logs were captured');
    }
    
    ws1.close();
    ws2.close();
}

finalLogTest().catch(console.error);