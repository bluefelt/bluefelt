const WebSocket = require('ws');

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function testTieDetection() {
    console.log('Testing Tic-Tac-Toe Tie Detection...\n');
    
    // Create lobby
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    const lobby = await createResponse.json();
    console.log(`Created lobby: ${lobby.id}`);
    
    // Connect players
    const p1 = new WebSocket(`ws://localhost:8000/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const p2 = new WebSocket(`ws://localhost:8000/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    let gameStarted = false;
    let tableId = null;
    let gameEnded = false;
    let tieDetected = false;
    
    // Set up message handlers
    p1.on('message', (data) => {
        const msg = JSON.parse(data);
        if (msg.type === 'gameStarted') {
            gameStarted = true;
            tableId = msg.gameInstanceId;
            console.log('Game started!');
        }
        if (msg.type === 'gameUpdate' && msg.patches) {
            for (const patch of msg.patches) {
                if (patch.path === '/game/gameStatus' && patch.value?.state === 'ended' && patch.value?.tie === true) {
                    tieDetected = true;
                    gameEnded = true;
                    console.log('TIE DETECTED!');
                }
            }
        }
    });
    
    // Wait for connections
    await sleep(1000);
    
    // Start game
    p1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for game to start
    await sleep(1000);
    
    if (!gameStarted) {
        console.error('Game failed to start!');
        return;
    }
    
    // Play moves that result in a tie
    const moves = [
        { player: p1, location: '/zones/board/cells/0/0', entity: 'mark_p1' }, // X
        { player: p2, location: '/zones/board/cells/0/1', entity: 'mark_p2' }, // O
        { player: p1, location: '/zones/board/cells/0/2', entity: 'mark_p1' }, // X
        { player: p2, location: '/zones/board/cells/1/0', entity: 'mark_p2' }, // O
        { player: p1, location: '/zones/board/cells/1/1', entity: 'mark_p1' }, // X
        { player: p2, location: '/zones/board/cells/2/0', entity: 'mark_p2' }, // O
        { player: p1, location: '/zones/board/cells/1/2', entity: 'mark_p1' }, // X
        { player: p2, location: '/zones/board/cells/2/2', entity: 'mark_p2' }, // O
        { player: p1, location: '/zones/board/cells/2/1', entity: 'mark_p1' }, // X - This should trigger tie
    ];
    
    console.log('\nPlaying moves for tie game:');
    console.log('X O X');
    console.log('O X X');
    console.log('O X O');
    console.log();
    
    for (let i = 0; i < moves.length; i++) {
        const move = moves[i];
        console.log(`Move ${i+1}: ${move.entity} at ${move.location.split('/').pop()}`);
        
        move.player.send(JSON.stringify({
            action: 'gameAction',
            gameInstanceId: tableId,
            gameAction: {
                action: 'placeMark',
                location: move.location,
                entity: move.entity
            }
        }));
        
        await sleep(500);
        
        if (gameEnded) {
            break;
        }
    }
    
    // Wait a bit more to ensure all messages are processed
    await sleep(1000);
    
    console.log('\nResult:');
    console.log(`Game ended: ${gameEnded}`);
    console.log(`Tie detected: ${tieDetected}`);
    
    if (tieDetected) {
        console.log('\n✅ TIE DETECTION WORKING!');
    } else {
        console.log('\n❌ TIE DETECTION FAILED!');
    }
    
    // Cleanup
    p1.close();
    p2.close();
    
    // Delete lobby
    await fetch(`http://localhost:8000/api/lobbies/${lobby.id}`, { method: 'DELETE' });
}

// Wait for server to be ready
setTimeout(() => {
    testTieDetection().catch(console.error);
}, 2000);