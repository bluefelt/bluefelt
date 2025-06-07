// Test to debug Connect Four board structure
const WebSocket = require('ws');

async function debugConnectFourBoard() {
    console.log('🔍 Debugging Connect Four board structure...\n');

    try {
        // Create lobby
        const response = await fetch('http://localhost:8000/api/lobbies', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ game_id: 'connect-four' })
        });
        const lobby = await response.json();
        console.log('✅ Created lobby:', lobby.id);

        // Create a Promise to handle WebSocket communication
        const gameStatePromise = new Promise((resolve, reject) => {
            const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
            const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
            
            let connectionsReady = 0;
            let gameStarted = false;

            const checkReady = () => {
                connectionsReady++;
                console.log(`📡 Connection ${connectionsReady}/2 ready`);
                if (connectionsReady === 2 && !gameStarted) {
                    gameStarted = true;
                    console.log('🎮 Starting game...');
                    ws1.send(JSON.stringify({ action: 'start_game' }));
                }
            };

            ws1.on('open', checkReady);
            ws2.on('open', checkReady);

            ws1.on('message', (data) => {
                const message = JSON.parse(data.toString());
                
                if (message.type === 'lobby_state') {
                    console.log('\n📊 Game State Received!');
                    console.log('==================');
                    
                    const gameState = message.state;
                    
                    // Examine board structure in detail
                    const board = gameState?.game?.zones?.board;
                    if (board && board.cells) {
                        console.log('\n🏁 Board Analysis:');
                        console.log('-------------------');
                        console.log('Board type:', board.type);
                        console.log('Board object keys:', Object.keys(board));
                        
                        if (Array.isArray(board.cells)) {
                            const rows = board.cells.length;
                            console.log(`📐 Board has ${rows} rows`);
                            
                            let totalCells = 0;
                            let actualCellsWithData = 0;
                            
                            board.cells.forEach((row, rowIdx) => {
                                if (Array.isArray(row)) {
                                    const cols = row.length;
                                    console.log(`   Row ${rowIdx}: ${cols} columns`);
                                    totalCells += cols;
                                    
                                    row.forEach((cell, colIdx) => {
                                        if (cell !== null && cell !== undefined) {
                                            actualCellsWithData++;
                                        }
                                        // Show first few cells for debugging
                                        if (rowIdx < 2 && colIdx < 4) {
                                            console.log(`     [${rowIdx}][${colIdx}]: ${JSON.stringify(cell)}`);
                                        }
                                    });
                                } else {
                                    console.log(`   Row ${rowIdx}: Not an array! ${typeof row}`);
                                }
                            });
                            
                            console.log(`\n📦 Total cells calculated: ${totalCells}`);
                            console.log(`📈 Cells with data: ${actualCellsWithData}`);
                            console.log(`🎯 Expected for Connect Four: 42 (6×7)`);
                            
                            if (totalCells === 42) {
                                console.log('✅ SUCCESS: Board has correct number of cells!');
                            } else {
                                console.log(`❌ ERROR: Expected 42 cells, found ${totalCells}`);
                            }
                            
                            // Check if the issue is the "19" count
                            if (totalCells === 19) {
                                console.log('\n🚨 FOUND THE 19 CELL ISSUE!');
                                console.log('This suggests the board is not being initialized properly.');
                                console.log('Possible causes:');
                                console.log('- Zone definition is wrong');
                                console.log('- Server state initialization is incorrect');
                                console.log('- Board data is corrupted during transmission');
                            }
                        } else {
                            console.log('❌ ERROR: board.cells is not an array!', typeof board.cells);
                        }
                    } else {
                        console.log('❌ ERROR: No board data found in game state');
                        console.log('Available zones:', Object.keys(gameState?.game?.zones || {}));
                    }
                    
                    // Check UI zones for metadata
                    const uiZones = gameState?.ui?.zones;
                    if (uiZones) {
                        console.log('\n🎨 UI Zone Metadata:');
                        console.log('---------------------');
                        const boardZone = uiZones.find(z => z.id === 'board');
                        if (boardZone) {
                            console.log('Board zone metadata:', JSON.stringify(boardZone, null, 2));
                        } else {
                            console.log('No board zone found in UI metadata');
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

            setTimeout(() => {
                console.log('⏰ Test timeout - closing connections');
                ws1.close();
                ws2.close();
                reject(new Error('Test timeout'));
            }, 10000);
        });

        await gameStatePromise;
        console.log('\n🎉 Debug completed successfully!');
        
    } catch (error) {
        console.error('\n💥 Debug failed:', error.message);
        process.exit(1);
    }
}

debugConnectFourBoard();