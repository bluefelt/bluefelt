const WebSocket = require('ws');
const assert = require('assert');

/**
 * Test that Go Fish properly provides actions to the active player
 */
async function testGoFishActionMap() {
    console.log('🎣 Testing Go Fish Action Map\n');
    
    // Create lobby
    const lobbyResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'go-fish' })
    });
    
    const lobby = await lobbyResponse.json();
    console.log(`✓ Created lobby: ${lobby.id}`);
    
    // Connect players
    const p1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const p2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    let gameState = {
        started: false,
        currentPlayer: null,
        currentPhase: null,
        actionMaps: {},
        lastError: null,
        ui: null,
        fullState: null
    };
    
    // Set up message handlers
    p1.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        console.log(`[P1 received] ${msg.type}${msg.tick ? ` (tick ${msg.tick})` : ''}`);
        
        if (msg.type === 'welcome') {
            gameState.ui = msg.ui;
            gameState.actionMaps.p1 = msg.ui?.actionMap?.p1 || {};
            gameState.fullState = msg.game;
            console.log('P1 initial action map:', JSON.stringify(gameState.actionMaps.p1, null, 2));
        }
        
        if (msg.type === 'gameStarted') {
            gameState.started = true;
            gameState.currentPlayer = msg.game?.currentPlayer;
            gameState.currentPhase = msg.game?.phases?.game;
            gameState.fullState = msg.game;
            console.log('✓ Game started');
            console.log(`  Current player: ${gameState.currentPlayer}`);
            console.log(`  Current phase: ${gameState.currentPhase}`);
        }
        
        if (msg.type === 'diff' && msg.patch) {
            // Log all patches for debugging
            console.log(`  Received ${msg.patch.length} patches:`);
            for (const patch of msg.patch) {
                console.log(`    ${patch.op} ${patch.path}${patch.value !== undefined ? ' = ' + JSON.stringify(patch.value).substring(0, 100) : ''}`);
            }
            
            // Track phase changes
            for (const patch of msg.patch) {
                if (patch.path === '/game/phases/game') {
                    gameState.currentPhase = patch.value;
                    console.log(`  Phase changed to: ${patch.value}`);
                }
                if (patch.path === '/game/currentPlayer') {
                    gameState.currentPlayer = patch.value;
                    console.log(`  Current player changed to: ${patch.value}`);
                }
                if (patch.path === '/ui/actionMap') {
                    gameState.actionMaps = patch.value || {};
                    console.log(`  Action map updated:`);
                    console.log(`    P1 actions: ${Object.keys(gameState.actionMaps.p1 || {}).length}`);
                    console.log(`    P2 actions: ${Object.keys(gameState.actionMaps.p2 || {}).length}`);
                } else if (patch.path === '/ui/actionMap/p1') {
                    // Handle individual player action map updates
                    if (!gameState.actionMaps) gameState.actionMaps = {};
                    gameState.actionMaps.p1 = patch.value || {};
                    console.log(`  P1 action map updated:`);
                    console.log(`    P1 actions: ${Object.keys(gameState.actionMaps.p1).length}`);
                } else if (patch.path === '/ui/actionMap/p2') {
                    // Handle individual player action map updates
                    if (!gameState.actionMaps) gameState.actionMaps = {};
                    gameState.actionMaps.p2 = patch.value || {};
                    console.log(`  P2 action map updated:`);
                    console.log(`    P2 actions: ${Object.keys(gameState.actionMaps.p2).length}`);
                    
                    // Show sample actions
                    if (gameState.actionMaps.p1 && Object.keys(gameState.actionMaps.p1).length > 0) {
                        const sampleKey = Object.keys(gameState.actionMaps.p1)[0];
                        console.log(`    Sample P1 action: ${sampleKey} -> ${JSON.stringify(gameState.actionMaps.p1[sampleKey])}`);
                    }
                }
                if (patch.path === '/game/selection/availableRanks') {
                    console.log(`  Available ranks updated: ${JSON.stringify(patch.value)}`);
                }
            }
        }
        
        if (msg.type === 'error') {
            gameState.lastError = msg.message;
            console.error(`❌ Error: ${msg.message}`);
        }
    });
    
    p2.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'welcome') {
            gameState.actionMaps.p2 = msg.ui?.actionMap?.p2 || {};
        }
    });
    
    // Wait for connections
    await new Promise(resolve => {
        let connected = 0;
        p1.on('open', () => {
            connected++;
            if (connected === 2) resolve();
        });
        p2.on('open', () => {
            connected++;
            if (connected === 2) resolve();
        });
    });
    
    console.log('✓ Both players connected\n');
    
    // Start game
    await new Promise(resolve => setTimeout(resolve, 500));
    p1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for game to start and initial phase processing
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Wait a bit more for auto actions to process
    console.log('\nWaiting for auto actions to process...');
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    console.log('\n📊 Game State Analysis:');
    console.log(`Current player: ${gameState.currentPlayer}`);
    console.log(`Current phase: ${gameState.currentPhase}`);
    console.log(`P1 available actions: ${Object.keys(gameState.actionMaps.p1 || {}).length}`);
    console.log(`P2 available actions: ${Object.keys(gameState.actionMaps.p2 || {}).length}`);
    
    // Test specific phases
    console.log('\n📍 Testing Phase-Specific Actions:');
    
    // Check if we're in selectingRank phase
    if (gameState.currentPhase === 'selectingRank') {
        console.log('\n✓ In selectingRank phase');
        
        // Check if selection object exists with available ranks
        console.log('\nChecking game state for available ranks:');
        if (gameState.fullState?.selection) {
            console.log('  selection:', JSON.stringify(gameState.fullState.selection, null, 2));
        } else {
            console.log('  No selection object in state');
        }
        
        // P1 should have rank selection actions
        const p1Actions = gameState.actionMaps.p1 || {};
        const rankActions = Object.entries(p1Actions).filter(([key, value]) => 
            key.includes('choice_p1') && value.action === 'selectRank'
        );
        
        console.log(`\nFound ${rankActions.length} rank selection actions for P1`);
        if (rankActions.length > 0) {
            console.log('Sample rank actions:');
            rankActions.slice(0, 3).forEach(([key, value]) => {
                console.log(`  ${key} -> ${JSON.stringify(value)}`);
            });
        }
        
        // Also check all actions
        console.log('\nAll P1 actions:');
        Object.entries(p1Actions).forEach(([key, value]) => {
            console.log(`  ${key} -> ${JSON.stringify(value)}`);
        });
        
        assert(rankActions.length > 0, 'P1 should have rank selection actions in selectingRank phase');
        
        // Try to select a rank
        if (rankActions.length > 0) {
            const firstRank = rankActions[0][1].rank;
            console.log(`\nSelecting rank: ${firstRank}`);
            p1.send(JSON.stringify({
                action: 'selectRank',
                args: {
                    rank: firstRank
                }
            }));
            
            // Wait for phase change and get the latest game state
            let phaseChanged = false;
            let attempts = 0;
            while (!phaseChanged && attempts < 5) {
                await new Promise(resolve => setTimeout(resolve, 500));
                attempts++;
                // gameState will be updated by the message handler
                if (gameState.currentPhase === 'selectingPlayer') {
                    phaseChanged = true;
                }
            }
        }
    }
    
    // Check if we're in selectingPlayer phase
    if (gameState.currentPhase === 'selectingPlayer') {
        console.log('\n✓ In selectingPlayer phase');
        
        // P1 should have player selection actions
        const p1Actions = gameState.actionMaps.p1 || {};
        const playerActions = Object.entries(p1Actions).filter(([key, value]) => 
            key.includes('choice_p1') && value.action === 'selectPlayer'
        );
        
        console.log(`Found ${playerActions.length} player selection actions for P1`);
        if (playerActions.length > 0) {
            console.log('Player selection actions:');
            playerActions.forEach(([key, value]) => {
                console.log(`  ${key} -> ${JSON.stringify(value)}`);
            });
        }
        
        assert(playerActions.length > 0, 'P1 should have player selection actions in selectingPlayer phase');
    }
    
    // Clean up
    p1.close();
    p2.close();
    
    console.log('\n' + '='.repeat(50));
    const hasActions = Object.keys(gameState.actionMaps.p1 || {}).length > 0;
    console.log(hasActions ? '✅ TEST PASSED - Actions are being offered' : '❌ TEST FAILED - No actions offered');
    console.log('='.repeat(50));
    
    process.exit(hasActions ? 0 : 1);
}

// Run the test
testGoFishActionMap().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});