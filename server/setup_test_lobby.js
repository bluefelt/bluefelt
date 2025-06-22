const WebSocket = require('ws');

// Set up a test lobby in movement phase that we can access via browser
async function setupTestLobby() {
    console.log('🎮 Setting up Three Men\'s Morris test lobby...');
    
    // Create lobby
    const lobbyResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'three-mens-morris' })
    });
    const lobby = await lobbyResponse.json();
    console.log('✓ Created lobby:', lobby.id);

    // Connect two players via WebSocket to set up the game
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player_id=alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player_id=bob&join=true`);
    
    let playersConnected = 0;
    let gameStarted = false;

    return new Promise((resolve, reject) => {
        const checkStart = () => {
            playersConnected++;
            if (playersConnected === 2 && !gameStarted) {
                gameStarted = true;
                
                setTimeout(() => {
                    console.log('🚀 Starting game...');
                    ws1.send(JSON.stringify({ action: 'start_game' }));
                    
                    setTimeout(() => {
                        console.log('⏩ Fast-forwarding to movement phase...');
                        fastForwardToMovement();
                    }, 1000);
                }, 500);
            }
        };

        ws1.on('open', () => {
            console.log('[Alice] Connected');
            checkStart();
        });
        
        ws2.on('open', () => {
            console.log('[Bob] Connected');
            checkStart();
        });

        ws1.on('message', (data) => {
            const msg = JSON.parse(data.toString());
            if (msg.type === 'diff') {
                const phasePatches = msg.patch.filter(p => p.path?.includes('/phases/'));
                if (phasePatches.some(p => p.value === 'movement')) {
                    console.log('✅ Movement phase reached!');
                    console.log(`\n🌐 BROWSER TEST URL: http://localhost:5173/lobby/${lobby.id}`);
                    console.log('🎮 Connect as "alice" to test movement phase');
                    console.log('📍 In the browser:');
                    console.log('   1. Go to the URL above');
                    console.log('   2. Enter "alice" as your name');  
                    console.log('   3. Try clicking on the pieces during movement phase');
                    console.log('   4. Check browser console for logs');
                    console.log('\n⏳ Keeping lobby alive for 5 minutes...');
                    
                    setTimeout(() => {
                        console.log('\n⏰ Test session ending...');
                        ws1.close();
                        ws2.close();
                        resolve();
                    }, 300000); // 5 minutes
                }
            }
        });

        function fastForwardToMovement() {
            const placements = [
                { player: ws1, pos: '/zones/board/cells/0/0', entity: 'piece_p1' },
                { player: ws2, pos: '/zones/board/cells/1/1', entity: 'piece_p2' },
                { player: ws1, pos: '/zones/board/cells/0/1', entity: 'piece_p1' },
                { player: ws2, pos: '/zones/board/cells/2/0', entity: 'piece_p2' },
                { player: ws1, pos: '/zones/board/cells/1/2', entity: 'piece_p1' },
                { player: ws2, pos: '/zones/board/cells/2/2', entity: 'piece_p2' }
            ];
            
            placements.forEach((placement, i) => {
                setTimeout(() => {
                    console.log(`📍 Placing ${placement.entity} at ${placement.pos}`);
                    placement.player.send(JSON.stringify({
                        action: 'placeToken',
                        args: {
                            target: placement.pos,
                            entity: placement.entity
                        }
                    }));
                }, i * 500);
            });
        }

        setTimeout(() => {
            reject(new Error('Test setup timeout'));
        }, 30000);
    });
}

setupTestLobby().catch(console.error);