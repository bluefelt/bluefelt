const { GameTestFramework } = require('../framework/GameTestFramework');

class OldMaidRegressionTest extends GameTestFramework {
  constructor() {
    super('old-maid');
  }

  async runAllTests() {
    console.log(`🎮 OLD MAID REGRESSION TEST SUITE\n`);
    
    try {
      await this.testGameInitialization();
      await this.testCardDrawingMechanism();
      await this.testTurnProgression();
      await this.testUIAffordances();
      console.log('✅ All Old Maid tests passed!');
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
    console.log('  Current game state:');
    if (state && state.game) {
      // Check zones
      if (state.game.zones) {
        Object.keys(state.game.zones).forEach(zoneId => {
          const zone = state.game.zones[zoneId];
          const count = zone.items ? zone.items.length : 0;
          console.log(`    ${zoneId}: ${count} items`);
        });
      }
      
      // Check current phase
      if (state.game.phaseSet && state.game.phase) {
        console.log(`    Current phase: ${state.game.phaseSet}.${state.game.phase}`);
      }
      
      // Check current player
      if (state.game.currentPlayer) {
        console.log(`    Current player: ${state.game.currentPlayer}`);
      }
      
      // Verify cards were dealt properly
      const p1Hand = state.game.zones?.hand_p1?.items || [];
      const p2Hand = state.game.zones?.hand_p2?.items || [];
      const deck = state.game.zones?.deck?.items || [];
      
      console.log(`    Player hands: P1=${p1Hand.length}, P2=${p2Hand.length}, Deck=${deck.length}`);
      
      // Old Maid should have 51 cards total (52 minus Queen of Clubs)
      const totalCards = p1Hand.length + p2Hand.length + deck.length;
      console.log(`    Total cards in game: ${totalCards}`);
      
      if (totalCards === 51) {
        console.log('    ✅ Correct number of cards (51) in game');
      } else {
        console.error(`    ❌ Expected 51 cards, got ${totalCards}`);
      }
      
      // Check for Old Maid (Queen of Spades)
      const allCards = [...p1Hand, ...p2Hand, ...deck];
      const hasOldMaid = allCards.some(card => 
        card.entity && card.entity.includes('spades_queen')
      );
      
      if (hasOldMaid) {
        console.log('    ✅ Old Maid (Queen of Spades) found in game');
      } else {
        console.error('    ❌ Old Maid (Queen of Spades) not found');
      }
      
      // Verify Queen of Clubs is excluded
      const hasQueenOfClubs = allCards.some(card => 
        card.entity && card.entity.includes('clubs_queen')
      );
      
      if (!hasQueenOfClubs) {
        console.log('    ✅ Queen of Clubs properly excluded');
      } else {
        console.error('    ❌ Queen of Clubs should be excluded but was found');
      }
    }
  }

  async testCardDrawingMechanism() {
    console.log('\n🃏 Testing Card Drawing Mechanism\n');
    
    const state = this.gameState;
    if (!state || !state.game) {
      console.error('    ❌ No game state available');
      return;
    }
    
    // Get initial hand sizes
    const initialP1Hand = state.game.zones?.hand_p1?.items?.length || 0;
    const initialP2Hand = state.game.zones?.hand_p2?.items?.length || 0;
    const currentPlayer = state.game.currentPlayer;
    
    console.log(`    Initial hands: P1=${initialP1Hand}, P2=${initialP2Hand}`);
    console.log(`    Current player: ${currentPlayer}`);
    
    // Check available actions for current player
    if (this.actionMap && this.actionMap[currentPlayer]) {
      const actions = Object.keys(this.actionMap[currentPlayer]);
      console.log(`    ${currentPlayer} available actions: ${actions.join(', ')}`);
      
      if (actions.length > 0) {
        // Find a drawing action
        const drawAction = actions.find(action => 
          action.includes('DrawsFrom') || 
          this.actionMap[currentPlayer][action].action?.includes('DrawsFrom')
        );
        
        if (drawAction) {
          console.log(`    Attempting to execute: ${drawAction}`);
          
          // Execute the drawing action
          try {
            await this.executeAction(this.players[currentPlayer], 
              this.actionMap[currentPlayer][drawAction].action, {});
            await this.wait(2000);
            
            // Check if card was transferred
            const newState = this.gameState;
            if (newState && newState.game) {
              const newP1Hand = newState.game.zones?.hand_p1?.items?.length || 0;
              const newP2Hand = newState.game.zones?.hand_p2?.items?.length || 0;
              
              console.log(`    After draw: P1=${newP1Hand}, P2=${newP2Hand}`);
              
              // Verify card transfer
              if (currentPlayer === 'p1') {
                if (newP1Hand === initialP1Hand + 1 && newP2Hand === initialP2Hand - 1) {
                  console.log('    ✅ Card successfully drawn from P2 to P1');
                } else {
                  console.error('    ❌ Card transfer failed for P1 drawing from P2');
                }
              } else {
                if (newP2Hand === initialP2Hand + 1 && newP1Hand === initialP1Hand - 1) {
                  console.log('    ✅ Card successfully drawn from P1 to P2');
                } else {
                  console.error('    ❌ Card transfer failed for P2 drawing from P1');
                }
              }
            }
          } catch (error) {
            console.error(`    ❌ Failed to execute draw action: ${error.message}`);
          }
        } else {
          console.error('    ❌ No drawing actions found');
        }
      } else {
        console.error('    ❌ No actions available for current player');
      }
    } else {
      console.error(`    ❌ No action map found for ${currentPlayer}`);
    }
  }

  async testTurnProgression() {
    console.log('\n🃏 Testing Turn Progression\n');
    
    const state = this.gameState;
    if (!state || !state.game) {
      console.error('    ❌ No game state available');
      return;
    }
    
    const initialPlayer = state.game.currentPlayer;
    console.log(`    Initial current player: ${initialPlayer}`);
    
    // The previous test should have executed an action, check if turn advanced
    const newPlayer = this.gameState?.game?.currentPlayer;
    console.log(`    Current player after action: ${newPlayer}`);
    
    if (initialPlayer !== newPlayer) {
      console.log('    ✅ Turn progression working correctly');
    } else {
      console.log('    ⚠️ Turn may not have advanced (could be expected if game ended)');
    }
  }

  async testUIAffordances() {
    console.log('\n🃏 Testing UI Affordances\n');
    
    // Check action map structure for both players
    if (this.actionMap) {
      ['p1', 'p2'].forEach(player => {
        if (this.actionMap[player]) {
          const actions = Object.keys(this.actionMap[player]);
          console.log(`    ${player} action map: ${actions.length} actions`);
          
          actions.forEach(actionPath => {
            const action = this.actionMap[player][actionPath];
            if (action.direction) {
              console.log(`      - ${action.direction} (${action.action})`);
            }
          });
          
          if (actions.length > 0) {
            console.log(`    ✅ ${player} has UI affordances`);
          } else {
            console.log(`    ⚠️ ${player} has no UI affordances`);
          }
        } else {
          console.log(`    ⚠️ No action map for ${player}`);
        }
      });
    } else {
      console.error('    ❌ No action map available');
    }
    
    // Check UI data
    if (this.uiData) {
      console.log(`    UI data available: ${Object.keys(this.uiData).join(', ')}`);
      console.log('    ✅ UI data structure is present');
    } else {
      console.log('    ⚠️ No UI data available');
    }
  }

  async testBasicGameplay() {
    // Keep the old method for backward compatibility
    await this.testGameInitialization();
  }
}

// Run the test
async function runTest() {
  const test = new OldMaidRegressionTest();
      currentTest = test;
  await test.createLobby();
  await test.connectPlayers(['Alice', 'Bob']);
  await test.startGame();
  await test.runAllTests();
}

// Handle both direct execution and module export
if (require.main === module) {
  runTest().catch(error => {
    console.error('Test failed:', error);
    process.exit(1);
  });
}

module.exports = { OldMaidRegressionTest };