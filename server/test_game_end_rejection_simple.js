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
    
    // Wait for game to process the win
    await sleep(1000);
    
    console.log('\n=== Testing moves after game end ===');
    
    // Try to make moves after game has ended
    const invalidMoves = [
        { player: ws2, location: '/zones/board/cells/2/0', name: 'Bob' },
        { player: ws1, location: '/zones/board/cells/2/1', name: 'Alice' },
    ];
    
    let rejectionCount = 0;
    let moveAcceptedAfterEnd = false;
    
    // Listen for action_rejected messages
    const checkRejection = (data) => {
        const msg = JSON.parse(data);
        if (msg.type === 'action_rejected') {
            rejectionCount++;
            console.log(`✅ Move rejected: ${msg.message || 'Action not allowed'}`);
        } else if (msg.type === 'patch') {
            // Check if any patches modified the board after game end
            msg.patches.forEach(patch => {
                if (patch.path && patch.path.startsWith('/game/zones/board/cells/2/')) {
                    console.log(`❌ ERROR: Board was modified after game end! Path: ${patch.path}`);
                    moveAcceptedAfterEnd = true;
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
        await sleep(500);
    }
    
    await sleep(500);
    
    // Check results
    console.log('\n=== Test Results ===');
    if (!moveAcceptedAfterEnd && rejectionCount > 0) {
        console.log(`✅ SUCCESS: Moves after game end were properly rejected!`);
        console.log(`   ${rejectionCount} move(s) were rejected`);
    } else if (moveAcceptedAfterEnd) {
        console.log(`❌ FAILURE: Game allowed moves after it ended!`);
    } else {
        console.log(`❌ FAILURE: Expected rejections but got none`);
    }
    
    // Clean up
    ws1.close();
    ws2.close();
}

// Run test
testGameEndRejection().catch(console.error);