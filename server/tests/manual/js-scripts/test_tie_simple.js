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
    let moveCount = 0;
    
    // Set up message handlers
    p1.on('message', (data) => {
        const msg = JSON.parse(data);
        if (msg.type === 'gameStarted') {
            gameStarted = true;
            tableId = msg.gameInstanceId;
            console.log('Game started!');
        }
        if (msg.type === 'gameUpdate' && msg.patches) {
            moveCount++;
            console.log(`Move ${moveCount} completed`);
            
            for (const patch of msg.patches) {
                if (patch.path === '/game/gameStatus' && patch.value?.state === 'ended' && patch.value?.tie === true) {
                    tieDetected = true;
                    gameEnded = true;
                    console.log('*** TIE DETECTED! ***');
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
        if (gameEnded) {
            console.log('Game already ended, stopping...');
            break;
        }
        
        const move = moves[i];
        const playerName = move.player === p1 ? 'Alice' : 'Bob';
        const [, , , , row, col] = move.location.split('/');
        console.log(`Move ${i+1}: ${playerName} (${move.entity}) at (${row},${col})`);
        
        move.player.send(JSON.stringify({
            action: 'gameAction',
            gameId: tableId,
            data: {
                action: 'placeMark',
                location: move.location,
                entity: move.entity
            }
        }));
        
        await sleep(500);
    }
    
    // Wait a bit more to ensure all messages are processed
    await sleep(1000);
    
    console.log('\nResult:');
    console.log(`Game ended: ${gameEnded}`);
    console.log(`Tie detected: ${tieDetected}`);
    console.log(`Total moves: ${moveCount}`);
    
    if (tieDetected && moveCount === 9) {
        console.log('\n✅ TIE DETECTION WORKING CORRECTLY!');
    } else {
        console.log('\n❌ TIE DETECTION FAILED!');
        console.log(`Expected: tie after 9 moves`);
        console.log(`Got: ${tieDetected ? 'tie' : 'no tie'} after ${moveCount} moves`);
    }
    
    // Cleanup
    p1.close();
    p2.close();
    
    // Delete lobby
    await fetch(`http://localhost:8000/api/lobbies/${lobby.id}`, { method: 'DELETE' });
}

testTieDetection().catch(console.error);