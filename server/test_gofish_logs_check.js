const WebSocket = require('ws');

async function testGoFishLogs() {
    console.log('Testing Go Fish game logs...\n');
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'go-fish' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect two players
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    const logs = [];
    
    // Collect logs from player 1
    ws1.on('message', (data) => {
        const msg = JSON.parse(data);
        
        if (msg.patches) {
            msg.patches.forEach(patch => {
                // Check for game log patches
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
    
    // Wait for game to start and cards to be dealt
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Make a move - Alice selects rank and asks Bob
    console.log('\nAlice asking Bob for rank...');
    ws1.send(JSON.stringify({
        action: 'selectRank',
        rank: 'A'  // Ask for Aces
    }));
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    ws1.send(JSON.stringify({
        action: 'selectPlayer',
        targetPlayer: 'p2'
    }));
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Summary
    console.log('\n=== SUMMARY ===');
    console.log('Total logs collected:', logs.length);
    
    const unresolvedLogs = logs.filter(log => 
        log.message && log.message.match(/\{[^}]+\}/)
    );
    
    if (unresolvedLogs.length > 0) {
        console.log('\n⚠️  LOGS WITH UNRESOLVED VARIABLES:');
        unresolvedLogs.forEach((log, i) => {
            console.log(`${i + 1}. ${log.message}`);
            const vars = log.message.match(/\{[^}]+\}/g);
            console.log('   Variables:', vars);
        });
    } else if (logs.length === 0) {
        console.log('⚠️  No game logs were captured!');
    } else {
        console.log('✅ No unresolved template variables found!');
        console.log('\nAll logs:');
        logs.forEach((log, i) => {
            console.log(`${i + 1}. [${log.timestamp}] ${log.message}`);
        });
    }
    
    // Close connections
    ws1.close();
    ws2.close();
}

testGoFishLogs().catch(console.error);