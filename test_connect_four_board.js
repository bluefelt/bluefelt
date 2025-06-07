const WebSocket = require('ws');

async function testConnectFourBoard() {
    console.log('🎮 Testing Connect Four board structure...\n');

    // Create lobby
    const response = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'connect-four' })
    });
    const lobby = await response.json();
    console.log('✅ Created lobby:', lobby.id);

    // Connect players via WebSocket
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);

    let gameState = null;

    return new Promise((resolve, reject) => {
        let connectionsReady = 0;
        
        const checkReady = () => {
            connectionsReady++;
            if (connectionsReady === 2) {
                // Start the game
                console.log('✅ Both players connected, starting game...');
                ws1.send(JSON.stringify({ action: 'start_game' }));
            }
        };

        ws1.on('open', checkReady);
        ws2.on('open', checkReady);

        ws1.on('message', (data) => {
            const message = JSON.parse(data.toString());
            
            if (message.type === 'lobby_state') {
                gameState = message.state;
                console.log('\n📊 Game State Received');
                console.log('==================');
                
                // Check board structure
                const board = gameState?.game?.zones?.board;
                if (board && board.cells) {
                    const rows = board.cells.length;
                    const cols = board.cells[0]?.length || 0;
                    const totalCells = rows * cols;
                    
                    console.log(`🏁 Board dimensions: ${rows} rows × ${cols} columns`);
                    console.log(`📦 Total cells: ${totalCells}`);
                    
                    if (totalCells === 42) {
                        console.log('✅ Correct! Connect Four should have 42 cells (6×7)');
                    } else {
                        console.log(`❌ ERROR: Expected 42 cells, got ${totalCells}`);
                    }
                    
                    // Count actual cell data
                    let actualCellCount = 0;
                    board.cells.forEach((row, rowIdx) => {
                        row.forEach((cell, colIdx) => {
                            actualCellCount++;
                            if (rowIdx < 3 && colIdx < 3) { // Show first few cells
                                console.log(`    Cell [${rowIdx}][${colIdx}]: ${cell}`);
                            }
                        });
                    });
                    
                    console.log(`🔢 Actual cell count: ${actualCellCount}`);
                    
                    // Check UI zones metadata
                    const uiZones = gameState?.ui?.zones;
                    if (uiZones) {
                        const boardZone = uiZones.find(z => z.id === 'board');
                        if (boardZone) {
                            console.log(`🎯 UI Zone metadata:`, JSON.stringify(boardZone, null, 2));
                        }
                    }
                }
                
                ws1.close();
                ws2.close();
                resolve(gameState);
            }
        });

        ws1.on('error', (error) => {
            console.error('❌ WebSocket 1 error:', error);
            reject(error);
        });

        ws2.on('error', (error) => {
            console.error('❌ WebSocket 2 error:', error);
            reject(error);
        });

        // Timeout after 10 seconds
        setTimeout(() => {
            ws1.close();
            ws2.close();
            reject(new Error('Test timeout'));
        }, 10000);
    });
}

// Run the test
testConnectFourBoard()
    .then(() => {
        console.log('\n🎉 Test completed successfully!');
        process.exit(0);
    })
    .catch((error) => {
        console.error('\n💥 Test failed:', error);
        process.exit(1);
    });