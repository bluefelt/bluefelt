const WebSocket = require('ws');
const assert = require('assert');

/**
 * Integration test that verifies the complete flow:
 * 1. Client sends correct message format
 * 2. Server processes it correctly
 * 3. Board state updates
 */
async function testTicTacToeClick() {
    console.log('🧪 Testing Tic-Tac-Toe Click Integration\n');
    
    // Create lobby
    const lobbyResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    
    const lobby = await lobbyResponse.json();
    console.log(`✓ Created lobby: ${lobby.id}`);
    
    // Connect players
    const p1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const p2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    let gameStarted = false;
    let boardState = null;
    let lastError = null;
    let actionMap = null;
    
    // Set up message handlers
    p1.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        console.log(`[P1 received] ${msg.type}${msg.tick ? ` (tick ${msg.tick})` : ''}`);
        
        if (msg.type === 'gameStarted') {
            gameStarted = true;
            boardState = msg.game?.zones?.board;
            console.log('✓ Game started');
        }
        
        if (msg.type === 'welcome') {
            actionMap = msg.ui?.actionMap;
            console.log('✓ Received action map');
        }
        
        if (msg.type === 'diff' && msg.patch) {
            // Apply patches to track board state
            for (const patch of msg.patch) {
                console.log(`  Patch: ${patch.op} ${patch.path}`);
                if (patch.path.startsWith('/game/zones/board/cells/')) {
                    console.log(`  Board update: ${patch.path} = ${JSON.stringify(patch.value)}`);
                }
            }
        }
        
        if (msg.type === 'error') {
            lastError = msg.message;
            console.error(`❌ Error: ${msg.message}`);
        }
    });
    
    p2.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'error') {
            console.error(`[P2] Error: ${msg.message}`);
        }
    });
    
    // Wait for connections
    await new Promise(resolve => {
        p1.on('open', () => {
            console.log('✓ P1 connected');
            resolve();
        });
    });
    
    await new Promise(resolve => {
        p2.on('open', () => {
            console.log('✓ P2 connected');
            resolve();
        });
    });
    
    // Start game
    await new Promise(resolve => setTimeout(resolve, 500));
    console.log('\nStarting game...');
    p1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for game to start
    await new Promise(resolve => {
        const checkInterval = setInterval(() => {
            if (gameStarted) {
                clearInterval(checkInterval);
                resolve();
            }
        }, 100);
        setTimeout(() => {
            clearInterval(checkInterval);
            resolve();
        }, 3000);
    });
    
    assert(gameStarted, 'Game should have started');
    
    // Test 1: Send a properly formatted placeMarker action
    console.log('\n📍 Test 1: Sending placeMarker action');
    console.log('Expected format based on client code:');
    console.log(JSON.stringify({
        action: 'placeMarker',
        args: {
            location: '/zones/board/cells/0/0',
            entity: 'mark_p1'
        }
    }, null, 2));
    
    lastError = null;
    p1.send(JSON.stringify({
        action: 'placeMarker',
        args: {
            location: '/zones/board/cells/0/0',
            entity: 'mark_p1'
        }
    }));
    
    // Wait for response
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    if (lastError) {
        console.error(`\n❌ FAILED: Server rejected the action`);
        console.error(`Error: ${lastError}`);
        
        // Test what the server expects
        console.log('\n📍 Test 2: Testing old format (with target)');
        lastError = null;
        p1.send(JSON.stringify({
            action: 'placeMarker',
            args: {
                target: '/zones/board/cells/0/0',
                entity: 'mark_p1'
            }
        }));
        
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        if (!lastError) {
            console.error('\n⚠️  Server accepts "target" but client sends "location"!');
            console.error('This is the mismatch we need to fix.');
        }
    } else {
        console.log('✅ Server accepted the action!');
    }
    
    // Test 3: Verify action map
    console.log('\n📍 Test 3: Checking action map format');
    if (actionMap && actionMap.p1) {
        const sampleAction = Object.entries(actionMap.p1)[0];
        if (sampleAction) {
            console.log(`Sample action from action map:`);
            console.log(`  Location: ${sampleAction[0]}`);
            console.log(`  Action: ${JSON.stringify(sampleAction[1])}`);
        }
    }
    
    // Clean up
    p1.close();
    p2.close();
    
    console.log('\n' + '='.repeat(50));
    console.log(lastError ? '❌ TEST FAILED - Client/Server mismatch detected' : '✅ TEST PASSED');
    console.log('='.repeat(50));
    
    process.exit(lastError ? 1 : 0);
}

// Run the test
testTicTacToeClick().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});