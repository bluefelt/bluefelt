const WebSocket = require('ws');

async function testThreeMensMorrisLogs() {
    console.log('Testing Three Men\'s Morris game logs...\n');
    
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
    
    const logs = [];
    let allMessages = [];
    
    // Collect logs from player 1
    ws1.on('message', (data) => {
        const msg = JSON.parse(data);
        allMessages.push(msg);
        
        // Check for errors
        if (msg.error) {
            console.log('\n❌ ERROR:', msg.error);
        }
        
        // Debug: Show all patches
        if (msg.patches && msg.patches.length > 0) {
            console.log('\n--- Patches received ---');
            msg.patches.forEach(patch => {
                console.log(`${patch.op} ${patch.path}`);
                // Check for game log patches - note the correct path is /ui/gameLog
                if (patch.path && patch.path.includes('/ui/gameLog')) {
                    console.log('\n=== GAME LOG PATCH ===');
                    console.log('Path:', patch.path);
                    console.log('Value:', JSON.stringify(patch.value, null, 2));
                    logs.push(patch.value);
                    
                    // Check for unresolved template variables
                    if (patch.value && patch.value.message) {
                        const unresolvedVars = patch.value.message.match(/\{[^}]+\}/g);
                        if (unresolvedVars) {
                            console.log('⚠️  UNRESOLVED VARIABLES FOUND:', unresolvedVars);
                        }
                    }
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
    
    // Start game
    ws1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait a bit for game to start
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Make some moves to generate logs
    console.log('\nMaking moves to generate logs...');
    
    // Alice places a piece - using the correct action ID
    console.log('Alice placing piece at position 0...');
    ws1.send(JSON.stringify({
        action: 'placeToken',  // Correct action ID from YAML
        target: '0'  // Three Men's Morris uses 'target' not 'position'
    }));
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Bob places a piece
    console.log('Bob placing piece at position 1...');
    ws2.send(JSON.stringify({
        action: 'placeToken',
        target: '1'
    }));
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Continue placing pieces
    console.log('Alice placing piece at position 3...');
    ws1.send(JSON.stringify({
        action: 'placeToken',
        target: '3'
    }));
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    console.log('Bob placing piece at position 4...');
    ws2.send(JSON.stringify({
        action: 'placeToken',
        target: '4'
    }));
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Try to form a mill
    console.log('Alice placing piece at position 6 (attempting mill)...');
    ws1.send(JSON.stringify({
        action: 'placeToken',
        target: '6'  // This should form a mill (0-3-6)
    }));
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Summary
    console.log('\n=== SUMMARY ===');
    console.log('Total logs collected:', logs.length);
    console.log('Total messages received:', allMessages.length);
    
    // Debug: Show all patches received
    console.log('\nAll patches received:');
    allMessages.forEach(msg => {
        if (msg.patches) {
            msg.patches.forEach(patch => {
                if (patch.path && patch.path.includes('Log')) {
                    console.log(`- ${patch.op} ${patch.path}`);
                }
            });
        }
    });
    
    const unresolvedLogs = logs.filter(log => 
        log.message && log.message.match(/\{[^}]+\}/)
    );
    
    if (unresolvedLogs.length > 0) {
        console.log('\n⚠️  LOGS WITH UNRESOLVED VARIABLES:');
        unresolvedLogs.forEach((log, i) => {
            console.log(`${i + 1}. ${log.message}`);
        });
    } else if (logs.length === 0) {
        console.log('⚠️  No game logs were captured!');
    } else {
        console.log('✅ No unresolved template variables found!');
    }
    
    // Close connections
    ws1.close();
    ws2.close();
}

testThreeMensMorrisLogs().catch(console.error);