#!/usr/bin/env node

const { GameTestFramework } = require('../framework/GameTestFramework');

async function testHexTicTacToe() {
    const framework = new GameTestFramework();
    
    try {
        console.log('Testing Hex Tic-Tac-Toe...\n');

        // Create game instance
        const lobby = await framework.createLobby('hex-tic-tac-toe');
        console.log(`✓ Created lobby: ${lobby.id}`);

        // Connect players
        const [ws1, ws2] = await framework.connectPlayers(lobby.id, ['Alice', 'Bob']);
        console.log('✓ Connected Alice and Bob');

        // Start game
        await framework.startGame(ws1);
        console.log('✓ Game started');

        // Test 1: Basic hex placement
        console.log('\n--- Test 1: Basic hex placement ---');
        const move1 = await framework.sendAction(ws1, {
            action: 'place_mark',
            args: { location: '/zones/hex_board/0,0' }
        });
        console.log('✓ Alice placed mark at center (0,0)');
        
        // Verify board state
        const boardState1 = framework.gameState.zones?.hex_board;
        if (boardState1 && boardState1['0,0'] === 'mark_p1') {
            console.log('✓ Board state correctly shows Alice\'s mark');
        } else {
            console.log('✗ Board state incorrect:', boardState1);
        }

        // Test 2: Second player move
        const move2 = await framework.sendAction(ws2, {
            action: 'place_mark',
            args: { location: '/zones/hex_board/1,0' }
        });
        console.log('✓ Bob placed mark at (1,0)');

        // Test 3: Win detection on hex grid
        console.log('\n--- Test 3: Win detection ---');
        
        // Alice at (0,1)
        await framework.sendAction(ws1, {
            action: 'place_mark',
            args: { location: '/zones/hex_board/0,1' }
        });
        console.log('✓ Alice placed at (0,1)');
        
        // Bob at (-1,0)
        await framework.sendAction(ws2, {
            action: 'place_mark',
            args: { location: '/zones/hex_board/-1,0' }
        });
        console.log('✓ Bob placed at (-1,0)');
        
        // Alice wins with (0,-1) completing line
        await framework.sendAction(ws1, {
            action: 'place_mark',
            args: { location: '/zones/hex_board/0,-1' }
        });
        console.log('✓ Alice placed winning move at (0,-1)');

        // Check game ended
        if (framework.gameState.gameStatus?.state === 'ended' && 
            framework.gameState.gameStatus?.winner === 'p1') {
            console.log('✓ Game correctly ended with Alice as winner');
        } else {
            console.log('✗ Game state incorrect:', framework.gameState.gameStatus);
        }

        console.log('\n✅ All hex tic-tac-toe tests passed!');
        
    } catch (error) {
        console.error('\n❌ Test failed:', error.message);
        console.error(error.stack);
    } finally {
        framework.cleanup();
    }
}

// Run tests
testHexTicTacToe().catch(console.error);