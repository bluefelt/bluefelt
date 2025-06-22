const { GameTestFramework } = require('../framework/GameTestFramework');
const assert = require('assert');

/**
 * Complete regression test suite for Crazy Eights
 * Tests game initialization, card playing, and suit selection
 */
class CrazyEightsRegressionTest extends GameTestFramework {
  constructor() {
    super('crazy-eights');
    this.handSizes = { p1: 0, p2: 0 };
    this.discardPileSize = 0;
    this.currentSuit = null;
  }

  processPatch(patches) {
    for (const patch of patches) {
      // Track hand sizes
      if (patch.path.match(/\/game\/zones\/hand_p(\d)\/items/)) {
        const player = patch.path.match(/hand_p(\d)/)[1];
        if (patch.op === 'add') {
          this.handSizes[`p${player}`]++;
        } else if (patch.op === 'remove') {
          this.handSizes[`p${player}`]--;
        }
      }
      
      // Track discard pile
      if (patch.path.match(/\/game\/zones\/discardPile\/items/)) {
        if (patch.op === 'add') {
          this.discardPileSize++;
        }
      }
      
      // Track current player
      if (patch.path === '/game/currentPlayer') {
        this.currentPlayer = patch.value;
      }
      
      // Track phase changes
      if (patch.path === '/game/phases/game') {
        this.currentPhase = patch.value;
      }
      
      // Track suit selection
      if (patch.path === '/game/currentSuit') {
        this.currentSuit = patch.value;
      }
      
      // Track game status
      if (patch.path === '/game/gameStatus') {
        this.gameStatus = patch.value;
        if (patch.value?.state === 'ended') {
          this.gameEnded = true;
        }
      }
      
      // Track action map
      if (patch.path === '/ui/actionMap') {
        this.actionMap = patch.value;
      }
    }
  }

  onGameStarted(msg) {
    super.onGameStarted(msg);
    
    // Initialize hand sizes from game state
    if (msg.game?.zones) {
      this.handSizes.p1 = msg.game.zones.hand_p1?.items?.length || 0;
      this.handSizes.p2 = msg.game.zones.hand_p2?.items?.length || 0;
      this.discardPileSize = msg.game.zones.discardPile?.items?.length || 0;
      this.topCard = msg.game.zones.discardPile?.items?.slice(-1)[0]?.entity;
    }
    
    console.log(`\n📊 Initial State:`);
    console.log(`  P1 hand: ${this.handSizes.p1} cards`);
    console.log(`  P2 hand: ${this.handSizes.p2} cards`);
    console.log(`  Discard pile: ${this.discardPileSize} cards`);
    console.log(`  Top card: ${this.topCard}`);
  }

  async runFullTest() {
    try {
      // Create lobby and connect players
      await this.createLobby();
      await this.connectPlayers(['Alice', 'Bob']);
      await this.startGame();
      
      // Wait a bit for game to initialize
      await this.wait(1000);
      
      console.log('\n🎮 Testing Game Initialization\n');
      
      // Add assertions for game initialization
      this.addAssertion('Initial hand sizes', () => {
        assert.strictEqual(this.handSizes.p1, 7, 'P1 should have 7 cards');
        assert.strictEqual(this.handSizes.p2, 7, 'P2 should have 7 cards');
      });
      
      this.addAssertion('Initial discard pile', () => {
        assert.strictEqual(this.discardPileSize, 1, 'Discard pile should have 1 card');
      });
      
      this.addAssertion('Initial game state', () => {
        assert.strictEqual(this.currentPlayer, 'p1', 'P1 should go first');
        assert.strictEqual(this.currentPhase, 'play', 'Should be in play phase');
      });
      
      this.addAssertion('Action map exists', () => {
        assert(this.actionMap, 'Action map should exist');
        assert(this.actionMap.p1, 'P1 should have actions');
        const p1Actions = Object.keys(this.actionMap.p1);
        assert(p1Actions.length > 0, 'P1 should have at least one action');
      });
      
      console.log('\n🃏 Testing Card Playing\n');
      
      // Find a playable card
      const p1Actions = Object.keys(this.actionMap.p1 || {});
      const playableCard = p1Actions.find(target => 
        this.actionMap.p1[target].action === 'playCard'
      );
      
      if (playableCard) {
        const action = this.actionMap.p1[playableCard];
        console.log(`  Playing ${action.entity}...`);
        
        const initialHandSize = this.handSizes.p1;
        const initialDiscardSize = this.discardPileSize;
        
        await this.executeAction('p1', 'playCard', action.args);
        await this.wait(500);
        
        // Check if it was an 8
        if (action.entity.includes('_8')) {
          this.addAssertion('8 card phase transition', () => {
            assert.strictEqual(this.currentPhase, 'chooseSuit', 'Should transition to chooseSuit phase');
          });
          
          console.log('  Played an 8, choosing suit...');
          
          // Choose a suit
          const suitActions = Object.keys(this.actionMap.p1 || {}).filter(t => 
            this.actionMap.p1[t].action === 'chooseSuit'
          );
          
          if (suitActions.length > 0) {
            const chooseSuit = this.actionMap.p1[suitActions[0]];
            await this.executeAction('p1', 'chooseSuit', chooseSuit.args);
            await this.wait(500);
          }
        } else {
          // Normal card play should advance turn
          this.addAssertion('Turn advancement', () => {
            assert.strictEqual(this.currentPlayer, 'p2', 'Turn should advance to P2');
          });
        }
        
        // Verify card moved
        this.addAssertion('Card movement', () => {
          assert.strictEqual(this.handSizes.p1, initialHandSize - 1, 'P1 hand should decrease by 1');
          assert.strictEqual(this.discardPileSize, initialDiscardSize + 1, 'Discard pile should increase by 1');
        });
      } else {
        console.log('  No playable cards, testing draw...');
        
        // Test drawing a card
        const drawAction = p1Actions.find(target => 
          this.actionMap.p1[target].action === 'drawCard'
        );
        
        if (drawAction) {
          const action = this.actionMap.p1[drawAction];
          const initialHandSize = this.handSizes.p1;
          
          await this.executeAction('p1', 'drawCard', action.args);
          await this.wait(500);
          
          this.addAssertion('Card draw', () => {
            assert.strictEqual(this.handSizes.p1, initialHandSize + 1, 'P1 hand should increase by 1');
            assert.strictEqual(this.currentPlayer, 'p2', 'Turn should advance to P2');
          });
        }
      }
      
      console.log('\n📝 Testing Game Logs\n');
      
      // Set up log expectations
      this.expectLog(/Dealing 7 cards/, 'Should log dealing cards to both players');
      this.expectLog(/Starting discard pile/, 'Should log starting discard pile');
      this.expectLog(/Game begins/, 'Should log game start');
      
      // Run assertions
      const assertionsPassed = await this.runAssertions();
      const logsPassed = this.validateLogs();
      
      if (assertionsPassed && logsPassed) {
        console.log('\n✅ All Crazy Eights tests passed!\n');
        return true;
      } else {
        console.log('\n❌ Some tests failed\n');
        return false;
      }
      
    } catch (error) {
      console.error('\n❌ Test failed:', error.message);
      return false;
    } finally {
      this.cleanup();
    }
  }
}

// Export for test runner
if (typeof module !== 'undefined' && module.exports) {
  module.exports = CrazyEightsRegressionTest;
}

// Run standalone if called directly
if (require.main === module) {
  const test = new CrazyEightsRegressionTest();
      currentTest = test;
  test.runFullTest().then(success => {
    process.exit(success ? 0 : 1);
  });
}