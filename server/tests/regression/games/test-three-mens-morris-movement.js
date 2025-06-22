const { GameTestFramework } = require('../framework/GameTestFramework');

/**
 * Regression test for Three Men's Morris piece movement
 * Ensures that multi-step move actions properly update the board state
 */
async function testThreeMensMorrisMovement() {
    const framework = new GameTestFramework('three-mens-morris');
    
    try {
        await framework.createLobby();
        await framework.connect(['Alice', 'Bob']);
        await framework.startGame();
        
        // Phase 1: Placement
        console.log('Testing placement phase...');
        
        // Place pieces in specific positions to set up a movement test
        // Alice places at (0,0), (1,0), (2,1)
        // Bob places at (0,1), (0,2), (1,2)
        const placements = [
            { player: 'Alice', row: 0, col: 0 },
            { player: 'Bob', row: 0, col: 1 },
            { player: 'Alice', row: 1, col: 0 },
            { player: 'Bob', row: 0, col: 2 },
            { player: 'Alice', row: 2, col: 1 },
            { player: 'Bob', row: 1, col: 2 }
        ];
        
        for (const placement of placements) {
            await framework.sendAction(placement.player, 'placeToken', {
                target: `/zones/board/cells/${placement.row}/${placement.col}`
            });
            await framework.waitForStateUpdate();
        }
        
        // Verify placement phase completed
        // Wait a bit for phase transition
        await framework.waitForStateUpdate();
        
        // Check logs to confirm phase transition
        const hasMovementPhase = framework.gameLogs.some(log => 
            log.includes('entering movement phase')
        );
        framework.assert(hasMovementPhase, 
            'Movement phase transition should be logged');
        
        // Phase 2: Movement
        console.log('Testing movement phase...');
        
        // Get initial board state
        const boardBefore = await framework.getBoardState();
        console.log('Board before move:');
        framework.printBoard(boardBefore);
        
        // Test multi-step movement
        // Alice should move piece from (0,0) to (1,1)
        console.log('Initiating multi-step move...');
        
        // Step 1: Initiate move
        await framework.sendAction('Alice', 'movePiece', {});
        await framework.waitForStateUpdate();
        
        // Verify multi-step state exists
        const multiStepState1 = await framework.getMultiStepState('p1');
        framework.assert(multiStepState1 !== null, 'Multi-step state should exist');
        framework.assert(multiStepState1.currentStepId === 'selectPiece', 
            `Expected step selectPiece, got: ${multiStepState1.currentStepId}`);
        
        // Step 2: Select piece at (0,0)
        const pieceLocation = '/zones/board/cells/0/0';
        const selectAction = multiStepState1.stepActionMap[pieceLocation];
        framework.assert(selectAction !== undefined, 
            `No action available for piece at ${pieceLocation}`);
        
        await framework.sendAction('Alice', selectAction.action, selectAction.args);
        await framework.waitForStateUpdate();
        
        // Verify moved to destination selection
        const multiStepState2 = await framework.getMultiStepState('p1');
        framework.assert(multiStepState2.currentStepId === 'selectDestination',
            `Expected step selectDestination, got: ${multiStepState2.currentStepId}`);
        
        // Step 3: Select destination at (1,1)
        const destLocation = '/zones/board/cells/1/1';
        const destAction = multiStepState2.stepActionMap[destLocation];
        framework.assert(destAction !== undefined,
            `No action available for destination at ${destLocation}`);
        
        await framework.sendAction('Alice', destAction.action, destAction.args);
        await framework.waitForStateUpdate();
        
        // Step 4: Confirm the move
        const multiStepState3 = await framework.getMultiStepState('p1');
        if (multiStepState3 && multiStepState3.requiresConfirmation) {
            console.log('Confirming move...');
            console.log('Multi-step state:', JSON.stringify(multiStepState3, null, 2));
            const actionId = multiStepState3.actionId || 'movePiece';
            console.log(`Using action ID: ${actionId}`);
            
            await framework.sendAction('Alice', 'multiStepConfirm', {
                actionId: actionId,
                confirmed: true
            });
            await framework.waitForStateUpdate();
        }
        
        // Verify the move completed
        const boardAfter = await framework.getBoardState();
        console.log('Board after move:');
        framework.printBoard(boardAfter);
        
        // Critical assertions: piece should have moved
        framework.assert(boardAfter[0][0] === null || boardAfter[0][0].entity === undefined,
            `Cell (0,0) should be empty after move, but contains: ${JSON.stringify(boardAfter[0][0])}`);
        framework.assert(boardAfter[1][1] && boardAfter[1][1].entity === 'piece_p1',
            `Cell (1,1) should contain piece_p1 after move, but contains: ${JSON.stringify(boardAfter[1][1])}`);
        
        // Verify multi-step state was cleared
        const multiStepStateAfter = await framework.getMultiStepState('p1');
        framework.assert(multiStepStateAfter === null || multiStepStateAfter === undefined,
            'Multi-step state should be cleared after completion');
        
        // Additional test: Verify patches were sent correctly
        const recentPatches = framework.getRecentPatches(10);
        const boardPatches = recentPatches.filter(p => 
            p.path && p.path.includes('/game/zones/board/cells')
        );
        
        // Should have at least 2 board patches (remove from source, add to destination)
        framework.assert(boardPatches.length >= 2,
            `Expected at least 2 board patches, but got ${boardPatches.length}`);
        
        // Verify patches have proper paths (not formatted coordinates)
        for (const patch of boardPatches) {
            framework.assert(!patch.path.includes('(') && !patch.path.includes(')'),
                `Patch path should not contain formatted coordinates: ${patch.path}`);
        }
        
        console.log('✓ All movement tests passed!');
        
    } catch (error) {
        console.error('Test failed:', error);
        process.exit(1);
    } finally {
        await framework.cleanup();
    }
}

// Run the test
testThreeMensMorrisMovement();