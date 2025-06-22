const WebSocket = require('ws');

async function test() {
    // Create lobby and connect players
    const res = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    const lobby = await res.json();
    
    const p1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const p2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    let initialBoard = null;
    let updatedBoard = null;
    let fullState = null;
    
    p1.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        
        if (msg.type === 'state') {
            fullState = msg.state;
            if (msg.state.zones && msg.state.zones.board) {
                initialBoard = JSON.stringify(msg.state.zones.board);
                console.log('Initial board state captured');
            }
        }
        
        if (msg.type === 'patch') {
            msg.patches.forEach(patch => {
                if (patch.path === '/zones/board' && patch.value) {
                    updatedBoard = JSON.stringify(patch.value);
                    console.log('\n✅ Board was updated via patch!');
                }
            });
        }
    });
    
    // Wait for connections and start game
    await new Promise(resolve => setTimeout(resolve, 500));
    p1.send(JSON.stringify({ action: 'start_game' }));
    await new Promise(resolve => setTimeout(resolve, 500));
    
    console.log('\n📋 Initial board state:');
    if (fullState && fullState.zones && fullState.zones.board) {
        console.log(JSON.stringify(fullState.zones.board, null, 2));
    }
    
    // Make a move
    console.log('\n🎮 Making move at 0,0...');
    p1.send(JSON.stringify({
        action: 'placeMarker',
        args: {
            location: '/zones/board/cells/0/0'
        }
    }));
    
    // Wait for update
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    if (updatedBoard) {
        console.log('\n📋 Updated board state:');
        console.log(JSON.parse(updatedBoard));
        
        // Check if cell 0,0 has been updated
        const board = JSON.parse(updatedBoard);
        if (board.cells && board.cells[0] && board.cells[0][0]) {
            console.log('\n✅ Cell [0,0] now contains:', board.cells[0][0]);
            if (board.cells[0][0].entity) {
                console.log('Entity:', board.cells[0][0].entity);
            }
        } else {
            console.log('\n❌ Cell [0,0] is still empty!');
        }
    } else {
        console.log('\n❌ No board update received!');
    }
    
    p1.close();
    p2.close();
}

test().catch(console.error);