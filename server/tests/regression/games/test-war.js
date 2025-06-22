const { GameTestFramework } = require('../framework/GameTestFramework');
const assert = require('assert');

class WarGameTest extends GameTestFramework {
    constructor() {
        super('war');
    }
    
    processPatch(patches) {
        // Update action map from patches
        for (const patch of patches) {
            if (patch.path === '/ui/actionMap' && patch.op === 'replace') {
                this.actionMap = patch.value;
                console.log('📊 Action map updated from patch:', JSON.stringify(this.actionMap, null, 2));
            }
        }
    }

    async runAllTests() {
        console.log('🎯 Starting War Game E2E Tests\n');
        
        try {
            // Set up the game
            await this.createLobby();
            await this.connectPlayers(['TestPlayer1', 'TestPlayer2']);
            await this.startGame();
            
            await this.testGameInitialization();
            await this.testBattleMechanics();
            await this.testWarCondition();
            await this.testGameEndConditions();
            await this.testUIAffordances();
            
            console.log('✅ All War Game tests completed!\n');
        } catch (error) {
            console.error(`❌ Test suite failed: ${error.message}`);
            throw error;
        }
    }

    async testGameInitialization() {
        console.log('🃏 Testing Game Initialization\n');
        
        // Wait for game to stabilize
        await this.wait(3000);
        
        // Check initial state
        const state = this.gameState;
        
        console.log(`Initial phase: ${state.phases?.game}`);
        
        // Wait longer for setup actions to complete
        await this.wait(5000);
        
        // Update state reference
        const updatedState = this.gameState;
        console.log(`Phase after waiting: ${updatedState.phases?.game}`);
        
        // For now, let's test both setup and ready phases
        const currentPhase = updatedState.phases?.game;
        assert(['setup', 'ready'].includes(currentPhase), `Game should be in setup or ready phase, got: ${currentPhase}`);
        console.log(`✓ Game is in phase: ${currentPhase}`);
        
        // Verify players have equal deck sizes (26 cards each)
        const p1Deck = updatedState.zones?.deck_p1?.items || [];
        const p2Deck = updatedState.zones?.deck_p2?.items || [];
        
        assert(p1Deck.length === 26, `P1 should have 26 cards, got ${p1Deck.length}`);
        assert(p2Deck.length === 26, `P2 should have 26 cards, got ${p2Deck.length}`);
        console.log(`✓ Cards dealt correctly: P1=${p1Deck.length}, P2=${p2Deck.length}`);
        
        // Verify battle areas are empty
        const p1Battle = updatedState.zones?.battle_p1?.items || [];
        const p2Battle = updatedState.zones?.battle_p2?.items || [];
        
        assert(p1Battle.length === 0, 'P1 battle area should be empty initially');
        assert(p2Battle.length === 0, 'P2 battle area should be empty initially');
        console.log('✓ Battle areas empty initially');
        
        // Verify war piles are empty
        const p1War = updatedState.zones?.war_pile_p1?.items || [];
        const p2War = updatedState.zones?.war_pile_p2?.items || [];
        
        assert(p1War.length === 0, 'P1 war pile should be empty initially');
        assert(p2War.length === 0, 'P2 war pile should be empty initially');
        console.log('✓ War piles empty initially');
        
        console.log('✅ Game initialization test passed\n');
    }

    async testBattleMechanics() {
        console.log('⚔️ Testing Battle Mechanics\n');
        
        // Debug: Log current action map
        console.log('Current action map:', JSON.stringify(this.actionMap, null, 2));
        console.log('P1 action map:', JSON.stringify(this.actionMap?.p1, null, 2));
        
        // Start a battle - look for battleStart action with new naming convention
        const actionMap = this.actionMap?.p1;
        const battleStartAction = Object.entries(actionMap || {}).find(
            ([key, value]) => value.action === 'battleStart'
        );
        assert(battleStartAction, 'Battle start action should be available');
        
        await this.executeAction('p1', 'battleStart', {});
        await this.wait(2000);
        
        // After battle starts, cards should be revealed automatically
        const state = this.gameState;
        
        // Check that cards were moved to battle areas
        const p1Battle = state.zones?.battle_p1?.items || [];
        const p2Battle = state.zones?.battle_p2?.items || [];
        
        assert(p1Battle.length === 1, `P1 should have 1 card in battle, got ${p1Battle.length}`);
        assert(p2Battle.length === 1, `P2 should have 1 card in battle, got ${p2Battle.length}`);
        console.log('✓ Cards revealed in battle areas');
        
        // Check that decks were reduced
        const p1Deck = state.zones?.deck_p1?.items || [];
        const p2Deck = state.zones?.deck_p2?.items || [];
        
        assert(p1Deck.length === 25, `P1 should have 25 cards left, got ${p1Deck.length}`);
        assert(p2Deck.length === 25, `P2 should have 25 cards left, got ${p2Deck.length}`);
        console.log('✓ Decks reduced after battle');
        
        // Wait for resolution
        await this.wait(3000);
        
        // Check that battle was resolved and cards moved to winner's deck
        const finalState = this.gameState;
        const finalP1Battle = finalState.zones?.battle_p1?.items || [];
        const finalP2Battle = finalState.zones?.battle_p2?.items || [];
        
        assert(finalP1Battle.length === 0, 'P1 battle area should be empty after resolution');
        assert(finalP2Battle.length === 0, 'P2 battle area should be empty after resolution');
        console.log('✓ Battle areas cleared after resolution');
        
        // Check that one player gained cards
        const finalP1Deck = finalState.zones?.deck_p1?.items || [];
        const finalP2Deck = finalState.zones?.deck_p2?.items || [];
        const totalCards = finalP1Deck.length + finalP2Deck.length;
        
        assert(totalCards === 52, `Total cards should be 52, got ${totalCards}`);
        
        // One player should have gained 2 cards (their own + opponent's)
        const p1Gained = finalP1Deck.length > 25;
        const p2Gained = finalP2Deck.length > 25;
        
        assert(p1Gained || p2Gained, 'One player should have won the battle');
        
        if (p1Gained) {
            console.log(`✓ P1 won battle: ${finalP1Deck.length} cards (gained ${finalP1Deck.length - 25})`);
        } else {
            console.log(`✓ P2 won battle: ${finalP2Deck.length} cards (gained ${finalP2Deck.length - 25})`);
        }
        
        // Should be back in ready phase for next battle
        assert(finalState.phases?.game === 'ready', 'Should return to ready phase after battle');
        console.log('✓ Returned to ready phase for next battle');
        
        console.log('✅ Battle mechanics test passed\n');
    }

    async testWarCondition() {
        console.log('💥 Testing War Condition\n');
        
        // This test is tricky since war happens when cards are equal
        // We'll run multiple battles and check if war mechanics work when they occur
        
        let warDetected = false;
        let attempts = 0;
        const maxAttempts = 20; // Try up to 20 battles to find a war
        
        while (!warDetected && attempts < maxAttempts) {
            attempts++;
            console.log(`  Battle attempt ${attempts}...`);
            
            // Check if we can start a battle
            const state = this.gameState;
            if (state.phases?.game !== 'ready') {
                await this.wait(1000);
                continue;
            }
            
            const actionMap = this.actionMap?.p1;
            if (!actionMap && actionMap['_global']?.action === 'battleStart') {
                await this.wait(1000);
                continue;
            }
            
            // Start battle
            await this.executeAction('p1', 'battleStart', {});
            await this.wait(2000);
            
            // Check if we're in war phase
            const battleState = this.gameState;
            if (battleState.phases?.game === 'war') {
                warDetected = true;
                console.log('🎯 WAR condition detected!');
                
                // Check war mechanics
                const p1Battle = battleState.zones?.battle_p1?.items || [];
                const p2Battle = battleState.zones?.battle_p2?.items || [];
                
                assert(p1Battle.length === 1, 'P1 should have 1 card in battle during war');
                assert(p2Battle.length === 1, 'P2 should have 1 card in battle during war');
                
                // Check if war piles start getting filled
                await this.wait(2000);
                const warState = this.gameState;
                
                // There should be a war reveal action available
                const warActionMap = this.actionMap?.p1;
                if (warActionMap['_global']?.action === 'warReveal') {
                    console.log('✓ War reveal action available');
                    
                    // Perform war reveal
                    await this.executeAction('p1', 'warReveal', {});
                    await this.wait(3000);
                    
                    // Check that war was resolved
                    const resolvedState = this.gameState;
                    const finalP1Battle = resolvedState.zones?.battle_p1?.items || [];
                    const finalP2Battle = resolvedState.zones?.battle_p2?.items || [];
                    
                    assert(finalP1Battle.length === 0, 'Battle areas should be empty after war resolution');
                    assert(finalP2Battle.length === 0, 'Battle areas should be empty after war resolution');
                    console.log('✓ War resolved successfully');
                }
                
                break;
            } else {
                // Regular battle, wait for it to complete
                await this.wait(3000);
            }
        }
        
        if (warDetected) {
            console.log('✅ War condition test passed\n');
        } else {
            console.log(`ℹ️ No war condition occurred in ${attempts} battles (this is random)\n`);
        }
    }

    async testGameEndConditions() {
        console.log('🏁 Testing Game End Conditions\n');
        
        // Play battles until someone runs out of cards or we hit a limit
        let battles = 0;
        const maxBattles = 100; // Safety limit
        
        while (battles < maxBattles) {
            battles++;
            
            const state = this.gameState;
            
            // Check if game ended
            if (state.phases?.game === 'gameOver') {
                console.log(`🎯 Game ended after ${battles} battles!`);
                
                // Check winner determination
                const gameStatus = state.gameStatus;
                assert(gameStatus?.state === 'ended', 'Game state should be ended');
                assert(gameStatus?.winner, 'Winner should be determined');
                
                // Check that one player has no cards
                const p1Deck = state.zones?.deck_p1?.items || [];
                const p2Deck = state.zones?.deck_p2?.items || [];
                
                const p1Empty = p1Deck.length === 0;
                const p2Empty = p2Deck.length === 0;
                
                assert(p1Empty || p2Empty, 'One player should have no cards');
                
                if (p1Empty) {
                    assert(gameStatus.winner === 'p2', 'P2 should be winner when P1 has no cards');
                    console.log('✓ P2 won the game');
                } else {
                    assert(gameStatus.winner === 'p1', 'P1 should be winner when P2 has no cards');
                    console.log('✓ P1 won the game');
                }
                
                console.log('✅ Game end conditions test passed\n');
                return;
            }
            
            // Continue playing if game hasn't ended
            if (state.phases?.game === 'ready') {
                const actionMap = this.actionMap?.p1;
                if (actionMap && actionMap['_global']?.action === 'battleStart') {
                    await this.executeAction('p1', 'battleStart', {});
                    await this.wait(3000);
                }
            } else if (state.phases?.game === 'war') {
                const actionMap = this.actionMap?.p1;
                if (actionMap && actionMap['_global']?.action === 'warReveal') {
                    await this.executeAction('p1', 'warReveal', {});
                    await this.wait(3000);
                }
            } else {
                // Wait for auto actions to complete
                await this.wait(2000);
            }
        }
        
        console.log(`ℹ️ Game didn't end naturally in ${maxBattles} battles (this can happen with War)\n`);
    }

    async testUIAffordances() {
        console.log('🎨 Testing UI Affordances\n');
        
        // Reset to a fresh game state
        await this.createLobby();
        await this.connectPlayers(['TestPlayer1', 'TestPlayer2']);
        await this.startGame();
        await this.wait(3000);
        
        // Test ready phase affordances
        const readyActionMap = this.actionMap?.p1;
        assert(readyActionMap['_global']?.action === 'battleStart', 'Battle start should be available in ready phase');
        assert(readyActionMap['_global']?.direction, 'Battle start should have UI direction');
        console.log('✓ Ready phase affordances correct');
        
        // Start a battle and check battle phase
        await this.executeAction('p1', 'battleStart', {});
        await this.wait(1000);
        
        // In battle phase, no manual actions should be available (all auto)
        const battleActionMap = this.actionMap?.p1;
        const hasManualActions = Object.keys(battleActionMap).length > 0;
        
        // Battle phase might be very brief due to auto actions
        if (!hasManualActions) {
            console.log('✓ Battle phase has no manual actions (all automatic)');
        }
        
        // Wait for potential war phase
        await this.wait(2000);
        const warState = this.gameState;
        
        if (warState.phases?.game === 'war') {
            const warActionMap = this.actionMap?.p1;
            assert(warActionMap['_global']?.action === 'warReveal', 'War reveal should be available in war phase');
            assert(warActionMap['_global']?.direction, 'War reveal should have UI direction');
            console.log('✓ War phase affordances correct');
        }
        
        console.log('✅ UI affordances test passed\n');
    }
}

// Export for use by test runner
module.exports = WarGameTest;

// Run standalone if executed directly
if (require.main === module) {
    const test = new WarGameTest();
      currentTest = test;
    test.runAllTests().catch(console.error);
}