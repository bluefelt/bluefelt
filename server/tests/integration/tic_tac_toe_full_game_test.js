const WebSocket = require('ws');
const assert = require('assert');

/**
 * Full integration test that plays a complete tic-tac-toe game
 * and verifies all state changes
 */
async function testFullTicTacToeGame() {
    console.log('🎮 Testing Full Tic-Tac-Toe Game\n');
    
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
    
    let gameState = {
        started: false,
        board: null,
        currentPlayer: null,
        gameStatus: null,
        lastError: null,
        moveCount: 0
    };
    
    // Track board state
    function updateBoard(patches) {
        for (const patch of patches) {
            const match = patch.path.match(/\/game\/zones\/board\/cells\/(\d+)\/(\d+)/);
            if (match) {
                const [_, row, col] = match;
                if (!gameState.board) {
                    gameState.board = [[null, null, null], [null, null, null], [null, null, null]];
                }
                gameState.board[row][col] = patch.value?.entity || null;
                gameState.moveCount++;
                console.log(`  Board[${row}][${col}] = ${patch.value?.entity || 'empty'}`);
            }
            
            if (patch.path === '/game/currentPlayer') {
                gameState.currentPlayer = patch.value;
                console.log(`  Current player: ${patch.value}`);
            }
            
            if (patch.path === '/game/gameStatus') {
                gameState.gameStatus = patch.value;
                console.log(`  Game status: ${JSON.stringify(patch.value)}`);
            }
        }
    }
    
    // Print board
    function printBoard() {
        if (!gameState.board) return;
        console.log('\n  Current board:');
        for (let row = 0; row < 3; row++) {
            let line = '    ';
            for (let col = 0; col < 3; col++) {
                const cell = gameState.board[row][col];
                if (cell === 'mark_p1') line += 'X ';
                else if (cell === 'mark_p2') line += 'O ';
                else line += '- ';
            }
            console.log(line);
        }
        console.log();
    }
    
    // Set up message handlers
    p1.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        
        if (msg.type === 'gameStarted') {
            gameState.started = true;
            gameState.currentPlayer = msg.game?.currentPlayer;
            console.log('✓ Game started');
        }
        
        if (msg.type === 'diff' && msg.patch) {
            console.log(`\n[Move ${gameState.moveCount + 1}]`);
            updateBoard(msg.patch);
            printBoard();
        }
        
        if (msg.type === 'error') {
            gameState.lastError = msg.message;
            console.error(`❌ Error: ${msg.message}`);
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
    
    console.log('✓ Both players connected');
    
    // Start game
    await new Promise(resolve => setTimeout(resolve, 500));
    p1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for game to start
    await new Promise(resolve => {
        const checkInterval = setInterval(() => {
            if (gameState.started) {
                clearInterval(checkInterval);
                resolve();
            }
        }, 100);
        setTimeout(() => {
            clearInterval(checkInterval);
            resolve();
        }, 3000);
    });
    
    assert(gameState.started, 'Game should have started');
    
    // Helper to make a move
    async function makeMove(player, ws, row, col) {
        console.log(`\n📍 ${player} placing at (${row}, ${col})`);
        
        const prevMoveCount = gameState.moveCount;
        gameState.lastError = null;
        
        ws.send(JSON.stringify({
            action: 'placeMarker',
            args: {
                location: `/zones/board/cells/${row}/${col}`,
                entity: `mark_${player}`
            }
        }));
        
        // Wait for board update
        await new Promise(resolve => {
            const checkInterval = setInterval(() => {
                if (gameState.moveCount > prevMoveCount || gameState.lastError) {
                    clearInterval(checkInterval);
                    resolve();
                }
            }, 50);
            setTimeout(() => {
                clearInterval(checkInterval);
                resolve();
            }, 2000);
        });
        
        if (gameState.lastError) {
            throw new Error(gameState.lastError);
        }
        
        // Verify the move was placed
        assert.strictEqual(
            gameState.board[row][col], 
            `mark_${player}`,
            `Board[${row}][${col}] should be mark_${player}`
        );
    }
    
    // Play a game where P1 wins horizontally
    console.log('\n🎯 Playing game - P1 wins horizontally\n');
    
    try {
        await makeMove('p1', p1, 0, 0);  // X - -
        await makeMove('p2', p2, 1, 0);  // - - -
                                          // O - -
        
        await makeMove('p1', p1, 0, 1);  // X X -
        await makeMove('p2', p2, 1, 1);  // O O -
                                          // - - -
        
        await makeMove('p1', p1, 0, 2);  // X X X - P1 wins!
        
        // Wait for game end
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Verify game ended
        assert(gameState.gameStatus, 'Game status should be set');
        assert.strictEqual(gameState.gameStatus.state, 'ended', 'Game should be ended');
        assert.strictEqual(gameState.gameStatus.winner, 'p1', 'P1 should be winner');
        assert.strictEqual(gameState.gameStatus.tie, false, 'Should not be a tie');
        
        console.log('\n✅ Game completed successfully!');
        console.log(`Winner: ${gameState.gameStatus.winner}`);
        console.log(`Total moves: ${gameState.moveCount}`);
        
    } catch (error) {
        console.error('\n❌ Test failed:', error.message);
        printBoard();
        p1.close();
        p2.close();
        process.exit(1);
    }
    
    // Test that no more moves are allowed after game end
    console.log('\n📍 Testing move after game end...');
    const prevStatus = gameState.gameStatus;
    p2.send(JSON.stringify({
        action: 'placeMarker',
        args: {
            location: '/zones/board/cells/2/2',
            entity: 'mark_p2'
        }
    }));
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Status should not change
    assert.deepStrictEqual(gameState.gameStatus, prevStatus, 'Game status should not change after end');
    
    // Clean up
    p1.close();
    p2.close();
    
    console.log('\n' + '='.repeat(50));
    console.log('✅ ALL TESTS PASSED!');
    console.log('='.repeat(50));
    
    process.exit(0);
}

// Run the test
testFullTicTacToeGame().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});