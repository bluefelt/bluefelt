const WebSocket = require('ws');

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function testActionMapGeneration() {
    console.log('Testing action map generation for tic-tac-toe...');
    
    // Create lobby
    const response = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    
    const lobby = await response.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect Alice
    const alice = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    
    await new Promise((resolve) => {
        alice.on('open', () => {
            console.log('Alice connected');
            resolve();
        });
    });
    
    // Connect Bob
    const bob = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    await new Promise((resolve) => {
        bob.on('open', () => {
            console.log('Bob connected');
            resolve();
        });
    });
    
    // Set up message handlers to capture action maps
    let gameStarted = false;
    let initialActionMap = null;
    let tableId = null;
    
    alice.on('message', (data) => {
        const msg = JSON.parse(data);
        console.log('Alice received:', msg.type);
        
        if (msg.type === 'tableCreated') {
            tableId = msg.table.id;
            console.log('Table created:', tableId);
        }
        
        if (msg.type === 'gameStarted') {
            gameStarted = true;
            initialActionMap = msg.ui.actionMap;
            console.log('\n=== GAME STARTED ===');
            console.log('Initial action map for p1:', JSON.stringify(msg.ui.actionMap.p1, null, 2));
            
            // Count available actions
            const p1Actions = Object.keys(msg.ui.actionMap.p1 || {}).length;
            console.log(`\nP1 has ${p1Actions} available actions`);
            
            if (p1Actions === 9) {
                console.log('✅ SUCCESS: All 9 cells are available for placement!');
            } else {
                console.log('❌ FAILURE: Expected 9 available actions, got', p1Actions);
            }
        }
    });
    
    bob.on('message', (data) => {
        const msg = JSON.parse(data);
        console.log('Bob received:', msg.type);
    });
    
    await sleep(500);
    
    // Create table
    alice.send(JSON.stringify({
        action: 'createTable',
        bundleId: 'tic-tac-toe'
    }));
    
    await sleep(500);
    
    // Wait for table to be created
    await sleep(1000);
    
    // Bob joins table
    if (tableId) {
        bob.send(JSON.stringify({
            action: 'joinTable',
            tableId: tableId
        }));
        
        await sleep(500);
        
        // Start game
        alice.send(JSON.stringify({
            action: 'setReady',
            tableId: tableId,
            ready: true
        }));
        
        bob.send(JSON.stringify({
            action: 'setReady',
            tableId: tableId,
            ready: true
        }));
    } else {
        console.log('ERROR: No table ID available');
    }
    
    // Wait for game to start (countdown is 3 seconds)
    await sleep(4000);
    
    // Clean up
    alice.close();
    bob.close();
    
    // Delete lobby
    await fetch(`http://localhost:8000/api/lobbies/${lobby.id}`, {
        method: 'DELETE'
    });
    
    console.log('\nTest complete');
}

// Run the test
testActionMapGeneration().catch(console.error);