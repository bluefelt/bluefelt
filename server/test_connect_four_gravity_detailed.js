const WebSocket = require('ws');

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function testConnectFourGravity() {
    console.log('Testing Connect Four gravity mechanics...\n');
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'connect-four' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect players
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    let gameStarted = false;
    let turnCount = 0;
    
    // Set up message handlers
    const handleMessage = (playerName) => (data) => {
        const msg = JSON.parse(data);
        console.log(`\n${playerName} received:`, msg.type);
        
        if (msg.type === 'welcome') {
            console.log(`  Players in lobby:`, msg.state?.players || 'none');
        }
        
        if (msg.type === 'gameStarted') {
            console.log('  ✅ Game started!');
        }
        
        if (msg.type === 'diff' && msg.patch) {
            console.log(`  ${msg.patch.length} patches received`);
            
            for (const patch of msg.patch) {
                // Log all patches for debugging
                if (patch.path.includes('board')) {
                    console.log(`  📍 Board patch:`, patch.op, patch.path);
                    if (patch.value) {
                        console.log(`     Value:`, JSON.stringify(patch.value));
                    }
                }
                
                // Check for disc placement
                if (patch.path && patch.path.includes('/game/zones/board/cells/')) {
                    console.log('\n  🎯 DISC PLACEMENT DETECTED!');
                    console.log(`     Path: ${patch.path}`);
                    console.log(`     Value:`, JSON.stringify(patch.value));
                    
                    const match = patch.path.match(/\/cells\/(\d+)\/(\d+)$/);
                    if (match) {
                        const row = parseInt(match[1]);
                        const col = parseInt(match[2]);
                        console.log(`     Position: Row ${row}, Column ${col}`);
                        
                        if (row === 5) {
                            console.log('     ✅ Disc placed at BOTTOM ROW! Gravity is working!');
                        } else if (row === 4) {
                            console.log('     ✅ Disc stacked at row 4!');
                        }
                    }
                }
                
                // Check for turn changes
                if (patch.path === '/game/currentPlayer') {
                    console.log(`  🔄 Turn changed to: ${patch.value}`);
                }
                
                // Check for phase changes
                if (patch.path && patch.path.includes('phases')) {
                    console.log(`  📋 Phase change:`, patch.path, '→', patch.value);
                }
            }
        }
        
        if (msg.type === 'error') {
            console.error(`  ❌ Error:`, msg.message);
        }
    };
    
    ws1.on('message', handleMessage('Alice'));
    ws2.on('message', handleMessage('Bob'));
    
    ws2.on('message', (data) => {
        const msg = JSON.parse(data);
        if (msg.type === 'welcome' && !gameStarted) {
            gameStarted = true;
            setTimeout(() => {
                console.log('\n🎮 Starting game...');
                ws1.send(JSON.stringify({ action: 'start_game' }));
            }, 500);
        }
    });
    
    // Wait for connections and game start
    await sleep(1500);
    
    // Test gravity by dropping discs
    console.log('\n\n=== TEST 1: Drop disc in column 3 ===');
    const action1 = {
        action: 'dropDisc',
        targetColumn: 3
    };
    console.log('Sending action:', JSON.stringify(action1));
    ws1.send(JSON.stringify(action1));
    
    await sleep(500);
    
    console.log('\n\n=== TEST 2: Drop second disc in column 3 ===');
    ws2.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 3
    }));
    
    await sleep(500);
    
    console.log('\n\n=== TEST 3: Drop disc in column 0 ===');
    ws1.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 0
    }));
    
    await sleep(500);
    
    // Close connections
    ws1.close();
    ws2.close();
    
    console.log('\n\n✅ Connect Four gravity test completed!');
}

// Run the test
testConnectFourGravity().catch(console.error);