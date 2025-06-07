#!/usr/bin/env node

const WebSocket = require('ws');

async function createLobby() {
    const response = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'go-fish' })
    });
    const data = await response.json();
    return data.id;
}

class GameClient {
    constructor(playerName, playerId) {
        this.playerName = playerName;
        this.playerId = playerId;
        this.ws = null;
        this.state = {};
        this.logs = [];
    }

    connect(lobbyId) {
        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobbyId}/ws?player=${this.playerName}&join=true`);
            
            this.ws.on('open', () => {
                console.log(`[${this.playerName}] Connected`);
            });
            
            this.ws.on('message', (data) => {
                const message = JSON.parse(data.toString());
                
                if (message.type === 'welcome') {
                    this.state = message;
                    resolve();
                } else if (message.type === 'gameStarted') {
                    this.state = message;
                    if (message.ui?.gameLog) {
                        message.ui.gameLog.forEach(log => {
                            this.logs.push(log.message);
                            console.log(`📝 [${this.playerName}] Initial log: "${log.message}"`);
                        });
                    }
                } else if (message.type === 'diff') {
                    // Apply patches to state
                    this.applyPatches(message.patch);
                    
                    // Look for game log updates
                    const gameLogPatches = message.patch?.filter(p => p.path?.includes('/ui/gameLog'));
                    if (gameLogPatches) {
                        gameLogPatches.forEach(patch => {
                            if (patch.op === 'add' && patch.value?.message) {
                                this.logs.push(patch.value.message);
                                console.log(`📝 [${this.playerName}] Game Log: "${patch.value.message}"`);
                            }
                        });
                    }
                }
            });
            
            this.ws.on('error', (error) => {
                console.error(`[${this.playerName}] Error:`, error);
                reject(error);
            });
        });
    }
    
    applyPatches(patches) {
        patches.forEach(patch => {
            // Parse the path
            const pathParts = patch.path.split('/').filter(p => p !== '');
            
            if (pathParts.length === 0) return;
            
            // Navigate to the parent object
            let current = this.state;
            for (let i = 0; i < pathParts.length - 1; i++) {
                const part = pathParts[i];
                if (!current[part]) {
                    current[part] = {};
                }
                current = current[part];
            }
            
            // Apply the patch
            const lastPart = pathParts[pathParts.length - 1];
            if (patch.op === 'add' || patch.op === 'replace') {
                current[lastPart] = patch.value;
            } else if (patch.op === 'remove') {
                delete current[lastPart];
            }
            
            // Special handling for nested paths like /ui/actionMap/p1
            if (patch.path.startsWith('/ui/actionMap/')) {
                this.state.ui = this.state.ui || {};
                this.state.ui.actionMap = this.state.ui.actionMap || {};
                const player = pathParts[2];
                this.state.ui.actionMap[player] = patch.value;
            }
        });
    }
    
    async performAction() {
        if (!this.state.ui?.actionMap?.[this.playerId]) {
            return false;
        }
        
        const myActions = this.state.ui.actionMap[this.playerId];
        const actionKeys = Object.keys(myActions);
        
        if (actionKeys.length === 0) {
            return false;
        }
        
        // Try rank selection
        const rankAction = actionKeys.find(k => k.includes('/ranks/'));
        if (rankAction) {
            const action = myActions[rankAction];
            console.log(`\n🎮 [${this.playerName}] Selecting rank: ${action.rank}`);
            this.ws.send(JSON.stringify({
                action: 'selectRank',
                args: { rank: action.rank, player: this.playerId }
            }));
            return true;
        }
        
        // Try player selection
        const playerAction = actionKeys.find(k => k.includes('/players/'));
        if (playerAction) {
            const action = myActions[playerAction];
            console.log(`\n🎮 [${this.playerName}] Asking player: ${action.targetPlayer}`);
            this.ws.send(JSON.stringify({
                action: 'selectPlayer',
                args: { targetPlayer: action.targetPlayer, player: this.playerId }
            }));
            return true;
        }
        
        // Try drawing from pool
        const drawAction = actionKeys.find(k => k === '/zones/pool');
        if (drawAction) {
            console.log(`\n🎮 [${this.playerName}] Drawing from pool`);
            this.ws.send(JSON.stringify({
                action: 'drawCard',
                location: '/zones/pool'
            }));
            return true;
        }
        
        return false;
    }
    
    isMyTurn() {
        return this.state.game?.currentPlayer === this.playerId;
    }
    
    close() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

async function testFullGame() {
    try {
        console.log('=== Go Fish Full Game E2E Test ===\n');
        
        // Create lobby
        const lobbyId = await createLobby();
        console.log(`✅ Lobby created: ${lobbyId}`);
        
        // Connect players
        const alice = new GameClient('Alice', 'p1');
        const bob = new GameClient('Bob', 'p2');
        
        await alice.connect(lobbyId);
        await bob.connect(lobbyId);
        console.log('✅ Both players connected');
        
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Start game
        console.log('\n🎲 Starting game...');
        alice.ws.send(JSON.stringify({ action: 'start_game' }));
        
        // Wait for game to start and phases to process
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        // Play several turns
        let turnCount = 0;
        const maxTurns = 10;
        
        while (turnCount < maxTurns) {
            turnCount++;
            console.log(`\n=== Turn ${turnCount} ===`);
            
            // Check whose turn it is
            const aliceTurn = alice.isMyTurn();
            const bobTurn = bob.isMyTurn();
            
            if (aliceTurn) {
                console.log("It's Alice's turn");
                
                // Wait a bit for action map to be populated after phase processing
                let waitCount = 0;
                while (waitCount < 5) {
                    const actionMap = alice.state.ui?.actionMap?.[alice.playerId];
                    if (actionMap && Object.keys(actionMap).some(k => !actionMap[k].placeholder)) {
                        break; // Found real actions, not just placeholder
                    }
                    await new Promise(resolve => setTimeout(resolve, 500));
                    waitCount++;
                }
                
                const acted = await alice.performAction();
                if (!acted) {
                    console.log("Alice has no available actions");
                    console.log("Alice's actionMap:", JSON.stringify(alice.state.ui?.actionMap?.[alice.playerId], null, 2));
                    break;
                }
            } else if (bobTurn) {
                console.log("It's Bob's turn");
                const acted = await bob.performAction();
                if (!acted) {
                    console.log("Bob has no available actions");
                    break;
                }
            } else {
                console.log("No player's turn - game might be in transition");
            }
            
            // Wait for server to process
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            // Check if game ended
            if (alice.state.game?.gameStatus?.state === 'ended' || 
                bob.state.game?.gameStatus?.state === 'ended') {
                console.log('\n🏁 Game has ended!');
                break;
            }
        }
        
        // Analyze logs
        console.log('\n=== Game Log Analysis ===');
        console.log(`Total log entries: ${alice.logs.length}`);
        
        // Check for player names vs p1/p2
        const p1p2Logs = alice.logs.filter(log => /\bp1\b|\bp2\b/.test(log));
        const playerNameLogs = alice.logs.filter(log => /Alice|Bob/.test(log));
        
        console.log(`\n✅ Logs with player names: ${playerNameLogs.length}`);
        console.log(`❌ Logs with p1/p2: ${p1p2Logs.length}`);
        
        if (p1p2Logs.length > 0) {
            console.log('\n⚠️  Found logs with p1/p2 instead of player names:');
            p1p2Logs.forEach(log => console.log(`   - "${log}"`));
        }
        
        // Check for automatic action logs
        const goFishLogs = alice.logs.filter(log => log.includes("Go Fish"));
        const hasCardsLogs = alice.logs.filter(log => log.includes("has") && log.includes("gives"));
        const turnPassLogs = alice.logs.filter(log => log.includes("Turn passes"));
        
        console.log(`\n📊 Log types found:`);
        console.log(`   - Go Fish responses: ${goFishLogs.length}`);
        console.log(`   - Card transfers: ${hasCardsLogs.length}`);
        console.log(`   - Turn switches: ${turnPassLogs.length}`);
        
        // Test results
        console.log('\n=== Test Results ===');
        if (p1p2Logs.length === 0) {
            console.log('✅ All logs use player names correctly');
        } else {
            console.log('❌ Some logs still use p1/p2 instead of player names');
        }
        
        if (goFishLogs.length > 0 || hasCardsLogs.length > 0) {
            console.log('✅ Automatic action logs are being generated');
        } else {
            console.log('⚠️  No automatic action logs found');
        }
        
        if (turnPassLogs.length > 0) {
            console.log('✅ Turn switching logs are present');
        } else {
            console.log('⚠️  No turn switching logs found');
        }
        
        console.log('\n✅ Test complete!');
        alice.close();
        bob.close();
        
    } catch (error) {
        console.error('❌ Test failed:', error);
    }
}

// Run the test
testFullGame();