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
    let currentState = null;
    
    // Set up message handlers
    ws1.on('message', (data) => {
        const msg = JSON.parse(data);
        console.log('P1 received:', msg.type);
        
        if (msg.type === 'welcome') {
            currentState = msg.state;
            console.log('Game state keys:', Object.keys(msg.state || {}));
            if (msg.state?.game?.zones?.board) {
                console.log('Board exists, checking bottom row...');
                console.log('Bottom row (row 5):', JSON.stringify(msg.state.game.zones.board.cells?.[5], null, 2));
            }
        }
        
        if (msg.type === 'diff') {
            console.log('Number of patches:', msg.patch ? msg.patch.length : 0);
            
            if (msg.patch) {
                // Apply patches to state
                for (const patch of msg.patch) {
                    console.log('Patch:', JSON.stringify(patch));
                    
                    if (patch.path && patch.path.includes('/zones/board/cells/')) {
                        console.log('\n✓ Board update:', patch.op, patch.path, '→', JSON.stringify(patch.value));
                        
                        // Extract row and column from path
                        const match = patch.path.match(/\/cells\/(\d+)\/(\d+)$/);
                        if (match) {
                            const row = parseInt(match[1]);
                            const col = parseInt(match[2]);
                            console.log(`  Position: Row ${row}, Column ${col}`);
                            
                            if (row === 5) {
                                console.log('  → Disc placed at bottom row! Gravity working! ✓');
                            } else if (row === 4) {
                                console.log('  → Disc stacked at row 4! ✓');
                            }
                        }
                    }
                }
            }
        }
        
        if (msg.type === 'error') {
            console.error('Error:', msg.message);
        }
    });
    
    ws2.on('message', (data) => {
        const msg = JSON.parse(data);
        if (msg.type === 'welcome' && !gameStarted) {
            gameStarted = true;
            // Start the game
            ws1.send(JSON.stringify({ action: 'start_game' }));
        }
    });
    
    // Wait for connections
    await sleep(500);
    
    // Test gravity by dropping discs in column 3
    console.log('\nDropping disc in column 3...');
    ws1.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 3
    }));
    
    await sleep(200);
    
    console.log('\nDropping second disc in column 3...');
    ws2.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 3
    }));
    
    await sleep(200);
    
    // Test dropping in different columns
    console.log('\nDropping disc in column 0...');
    ws1.send(JSON.stringify({
        action: 'dropDisc',
        targetColumn: 0
    }));
    
    await sleep(200);
    
    // Close connections
    ws1.close();
    ws2.close();
    
    console.log('\n✅ Connect Four gravity test completed!');
}

// Run the test
testConnectFourGravity().catch(console.error);