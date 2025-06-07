const WebSocket = require('ws');

const SERVER_URL = 'ws://localhost:8000';
const API_URL = 'http://localhost:8000/api';

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function createLobby(gameId) {
    const response = await fetch(`${API_URL}/lobbies`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: gameId })
    });
    return response.json();
}

async function testGameEndRejection() {
    console.log('=== Testing Tic-Tac-Toe Game End Rejection ===\n');
    
    // Create lobby
    const lobby = await createLobby('tic-tac-toe');
    console.log(`Created lobby: ${lobby.id}`);
    
    // Connect players
    const ws1 = new WebSocket(`${SERVER_URL}/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`${SERVER_URL}/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    let gameState = null;
    let currentPlayer = null;
    let gameEnded = false;
    
    // Track messages
    const handleMessage = (data) => {
        const msg = JSON.parse(data);
        // console.log(`[DEBUG] Received message type: ${msg.type}`);
        if (msg.type === 'patch' && msg.patches) {
            // console.log(`[DEBUG] Patches:`, msg.patches.map(p => ({ path: p.path, value: p.value })));
            msg.patches.forEach(patch => {
                if (patch.path === '/game/currentPlayer') {
                    currentPlayer = patch.value;
                }
                if (patch.path === '/game/gameStatus' && patch.value?.state === 'ended') {
                    gameEnded = true;
                    console.log('\n🎮 GAME ENDED!');
                    console.log(`Winner: ${patch.value.winner || 'Tie'}`);
                }
            });
        }
        if (msg.type === 'welcome' && msg.state) {
            gameState = msg.state.game || msg.state;
        }
    };
    
    ws1.on('message', handleMessage);
    ws2.on('message', handleMessage);
    
    await sleep(500);
    
    // Start game
    console.log('\nStarting game...');
    ws1.send(JSON.stringify({ action: 'start_game' }));
    await sleep(500);
    
    // Play a winning sequence for Alice (X)
    const moves = [
        { player: ws1, location: '/zones/board/cells/0/0', name: 'Alice' }, // X
        { player: ws2, location: '/zones/board/cells/1/0', name: 'Bob' },   // O
        { player: ws1, location: '/zones/board/cells/0/1', name: 'Alice' }, // X
        { player: ws2, location: '/zones/board/cells/1/1', name: 'Bob' },   // O
        { player: ws1, location: '/zones/board/cells/0/2', name: 'Alice' }, // X - Wins!
    ];
    
    console.log('\nPlaying moves to create a win...');
    for (const move of moves) {
        console.log(`${move.name} places at ${move.location.split('/').slice(-2).join(',')}`);
        move.player.send(JSON.stringify({
            action: 'placeMarker',
            target: move.location
        }));
        await sleep(300);
    }
    
    // Wait for game to end
    await sleep(1000);
    
    if (!gameEnded) {
        console.log('\n❌ ERROR: Game should have ended but didn\'t!');
        ws1.close();
        ws2.close();
        return;
    }
    
    console.log('\n=== Testing moves after game end ===');
    
    // Try to make moves after game has ended
    const invalidMoves = [
        { player: ws2, location: '/zones/board/cells/2/0', name: 'Bob' },
        { player: ws1, location: '/zones/board/cells/2/1', name: 'Alice' },
    ];
    
    let rejectionCount = 0;
    
    // Listen for action_rejected messages
    const checkRejection = (data) => {
        const msg = JSON.parse(data);
        if (msg.type === 'action_rejected') {
            rejectionCount++;
            console.log(`✅ Move rejected: ${msg.message}`);
        } else if (msg.type === 'patch') {
            // Check if any patches modified the board after game end
            msg.patches.forEach(patch => {
                if (patch.path.startsWith('/game/zones/board/cells/')) {
                    console.log(`❌ ERROR: Board was modified after game end! Path: ${patch.path}`);
                }
            });
        }
    };
    
    ws1.on('message', checkRejection);
    ws2.on('message', checkRejection);
    
    // Try invalid moves
    for (const move of invalidMoves) {
        console.log(`\n${move.name} attempts to place at ${move.location.split('/').slice(-2).join(',')}`);
        move.player.send(JSON.stringify({
            action: 'placeMarker',
            target: move.location
        }));
        await sleep(300);
    }
    
    await sleep(500);
    
    // Check results
    console.log('\n=== Test Results ===');
    if (rejectionCount === invalidMoves.length) {
        console.log(`✅ SUCCESS: All ${rejectionCount} moves after game end were properly rejected!`);
    } else {
        console.log(`❌ FAILURE: Only ${rejectionCount} of ${invalidMoves.length} moves were rejected`);
    }
    
    // Clean up
    ws1.close();
    ws2.close();
}

// Run test
testGameEndRejection().catch(console.error);