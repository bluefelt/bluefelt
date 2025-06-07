const WebSocket = require('ws');

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function testConnectFourGravity() {
    console.log('=== Connect Four Gravity Test ===\n');
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'connect-four' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect both players
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    const boardUpdates = [];
    let gamePhase = null;
    let currentPlayer = null;
    
    // Track messages
    const setupHandler = (playerName, ws) => {
        ws.on('message', (data) => {
            const msg = JSON.parse(data);
            
            if (msg.type === 'error') {
                console.log(`\n❌ ERROR from ${playerName}:`, msg.message);
            }
            
            if (msg.type === 'gameLog') {
                console.log(`\n📝 Game Log from ${playerName}:`, msg.content);
            }
            
            if (msg.type === 'diff' && msg.patch) {
                console.log(`\n🔧 Diff from ${playerName}: ${msg.patch.length} patches`);
                for (const patch of msg.patch) {
                    console.log(`  Patch: ${patch.op} ${patch.path}`);
                    if (patch.value !== undefined && !patch.path.includes('actionMap')) {
                        console.log(`    Value: ${JSON.stringify(patch.value).substring(0, 100)}`);
                    }
                    
                    // Track phase
                    if (patch.path === '/game/phases/game') {
                        gamePhase = patch.value;
                        console.log(`  📋 Phase changed to: ${gamePhase}`);
                    }
                    
                    // Track current player
                    if (patch.path === '/game/currentPlayer') {
                        currentPlayer = patch.value;
                        console.log(`🔄 Current player: ${currentPlayer}`);
                    }
                    
                    // Track board updates
                    if (patch.path && patch.path.match(/\/game\/zones\/board\/cells\/\d+\/\d+/)) {
                        const match = patch.path.match(/\/cells\/(\d+)\/(\d+)$/);
                        if (match) {
                            const row = parseInt(match[1]);
                            const col = parseInt(match[2]);
                            const update = {
                                player: playerName,
                                row,
                                col,
                                value: patch.value,
                                path: patch.path
                            };
                            boardUpdates.push(update);
                            
                            console.log(`\n🎯 DISC PLACED!`);
                            console.log(`   Player: ${currentPlayer}`);
                            console.log(`   Position: Row ${row}, Column ${col}`);
                            console.log(`   Entity: ${patch.value.entity}`);
                            
                            if (row === 5) {
                                console.log(`   ✅ GRAVITY WORKING - Disc at bottom row!`);
                            } else if (row === 4) {
                                console.log(`   ✅ GRAVITY WORKING - Disc stacked at row 4!`);
                            } else if (row === 3) {
                                console.log(`   ✅ GRAVITY WORKING - Disc stacked at row 3!`);
                            }
                        }
                    }
                }
            }
        });
    };
    
    setupHandler('Alice', ws1);
    setupHandler('Bob', ws2);
    
    // Wait for both players to connect
    await sleep(1000);
    
    // Start game
    console.log('\n🎮 Starting game...');
    ws1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for game to start and phase to change
    await sleep(1000);
    
    // Test 1: Alice drops disc in column 3
    console.log('\n\n--- Test 1: Alice drops disc in column 3 ---');
    ws1.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 3
    }));
    await sleep(500);
    
    // Test 2: Bob drops disc in column 3 (should stack)
    console.log('\n\n--- Test 2: Bob drops disc in column 3 ---');
    ws2.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 3
    }));
    await sleep(500);
    
    // Test 3: Alice drops disc in column 3 (should stack higher)
    console.log('\n\n--- Test 3: Alice drops disc in column 3 ---');
    ws1.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 3
    }));
    await sleep(500);
    
    // Test 4: Bob drops disc in column 0
    console.log('\n\n--- Test 4: Bob drops disc in column 0 ---');
    ws2.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 0
    }));
    await sleep(500);
    
    // Summary
    console.log('\n\n=== SUMMARY ===');
    console.log(`Total board updates: ${boardUpdates.length}`);
    console.log('\nAll disc placements:');
    boardUpdates.forEach((update, i) => {
        console.log(`${i + 1}. Row ${update.row}, Col ${update.col}: ${update.value.entity}`);
    });
    
    // Verify gravity
    const col3Updates = boardUpdates.filter(u => u.col === 3);
    console.log(`\n✅ Column 3 updates: ${col3Updates.length}`);
    if (col3Updates.length >= 3) {
        console.log('   Expected rows (bottom-up): 5, 4, 3');
        console.log(`   Actual rows: ${col3Updates.map(u => u.row).join(', ')}`);
    }
    
    ws1.close();
    ws2.close();
    
    console.log('\n✅ Test completed!');
}

testConnectFourGravity().catch(console.error);