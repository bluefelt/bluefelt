const WebSocket = require('ws');

async function testGameLogs(gameId, actions) {
    console.log(`\n=== Testing ${gameId} ===`);
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: gameId })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect two players
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    const logs = [];
    let gameStarted = false;
    
    // Collect logs
    ws1.on('message', (data) => {
        const msg = JSON.parse(data);
        
        if (msg.patches) {
            msg.patches.forEach(patch => {
                if (patch.path && patch.path.includes('/ui/gameLog')) {
                    logs.push(patch.value);
                }
                if (patch.path === '/game/started' && patch.value === true) {
                    gameStarted = true;
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
    
    // Wait for game to start
    await new Promise(resolve => {
        const check = setInterval(() => {
            if (gameStarted) {
                clearInterval(check);
                resolve();
            }
        }, 100);
        setTimeout(() => {
            clearInterval(check);
            resolve();
        }, 2000);
    });
    
    // Execute test actions
    for (const action of actions) {
        console.log(`Executing: ${action.description}`);
        const ws = action.player === 'p1' ? ws1 : ws2;
        ws.send(JSON.stringify(action.data));
        await new Promise(resolve => setTimeout(resolve, 500));
    }
    
    // Wait a bit more for logs
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Check results
    console.log(`Total logs: ${logs.length}`);
    
    const unresolvedLogs = logs.filter(log => 
        log.message && log.message.match(/\{[^}]+\}/)
    );
    
    if (unresolvedLogs.length > 0) {
        console.log('❌ UNRESOLVED TEMPLATE VARIABLES:');
        unresolvedLogs.forEach(log => {
            console.log(`  - ${log.message}`);
            const vars = log.message.match(/\{[^}]+\}/g);
            console.log(`    Variables: ${vars.join(', ')}`);
        });
    } else {
        console.log('✅ No unresolved template variables');
    }
    
    // Close connections
    ws1.close();
    ws2.close();
    
    await new Promise(resolve => setTimeout(resolve, 500));
}

async function runAllTests() {
    // Test Tic Tac Toe
    await testGameLogs('tic-tac-toe', [
        {
            description: 'P1 places mark at center',
            player: 'p1',
            data: {
                action: 'placeMarker',
                location: '/zones/board/cells/1/1'
            }
        },
        {
            description: 'P2 places mark at corner',
            player: 'p2',
            data: {
                action: 'placeMarker',
                location: '/zones/board/cells/0/0'
            }
        }
    ]);
    
    // Test Three Men's Morris
    await testGameLogs('three-mens-morris', [
        {
            description: 'P1 places piece at position 0',
            player: 'p1',
            data: {
                action: 'placeToken',
                target: '/zones/board/0',
                entity: 'piece_p1'
            }
        },
        {
            description: 'P2 places piece at position 1',
            player: 'p2',
            data: {
                action: 'placeToken',
                target: '/zones/board/1',
                entity: 'piece_p2'
            }
        }
    ]);
    
    // Test Connect Four
    await testGameLogs('connect-four', [
        {
            description: 'P1 drops disc in column 3',
            player: 'p1',
            data: {
                action: 'dropDisc',
                column: 3
            }
        },
        {
            description: 'P2 drops disc in column 4',
            player: 'p2',
            data: {
                action: 'dropDisc',
                column: 4
            }
        }
    ]);
    
    // Test Go Fish
    await testGameLogs('go-fish', [
        {
            description: 'P1 selects rank A',
            player: 'p1',
            data: {
                action: 'selectRank',
                rank: 'A'
            }
        },
        {
            description: 'P1 asks P2',
            player: 'p1',
            data: {
                action: 'selectPlayer',
                targetPlayer: 'p2'
            }
        }
    ]);
    
    console.log('\n=== ALL TESTS COMPLETE ===');
}

runAllTests().catch(console.error);