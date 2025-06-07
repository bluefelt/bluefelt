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

function connectPlayer(lobbyId, playerName) {
    return new Promise((resolve, reject) => {
        const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobbyId}/ws?player=${playerName}&join=true`);
        
        ws.on('open', () => {
            console.log(`[${playerName}] Connected`);
        });
        
        ws.on('message', (data) => {
            const message = JSON.parse(data.toString());
            
            if (message.type === 'welcome') {
                console.log(`[${playerName}] Welcome received`);
                resolve(ws);
            } else if (message.type === 'gameStarted') {
                console.log(`[${playerName}] Game started`);
                // Log initial game log
                if (message.ui?.gameLog) {
                    console.log(`[${playerName}] Initial game log:`, message.ui.gameLog);
                }
            } else if (message.type === 'diff') {
                // Look for game log updates
                const gameLogPatches = message.patch?.filter(p => p.path?.includes('/ui/gameLog'));
                if (gameLogPatches && gameLogPatches.length > 0) {
                    gameLogPatches.forEach(patch => {
                        if (patch.op === 'add' && patch.value?.message) {
                            console.log(`📝 [${playerName}] Game Log: "${patch.value.message}"`);
                        }
                    });
                }
                
                // Look for phase changes
                const phasePatch = message.patch?.find(p => p.path?.includes('phase'));
                if (phasePatch) {
                    console.log(`[${playerName}] Phase change to:`, phasePatch.value);
                }
                
                // Look for action map to know when to act
                const actionMapPatch = message.patch?.find(p => p.path === '/ui/actionMap');
                if (actionMapPatch && playerName === 'Alice') {
                    const myActions = actionMapPatch.value.p1 || {};
                    const actionKeys = Object.keys(myActions);
                    if (actionKeys.length > 0) {
                        console.log(`[${playerName}] Available actions:`, actionKeys.length);
                        
                        // If we can select a rank, select one
                        const rankAction = actionKeys.find(k => k.includes('/ranks/'));
                        if (rankAction && ws.readyState === WebSocket.OPEN) {
                            const action = myActions[rankAction];
                            console.log(`[${playerName}] Selecting rank:`, action.rank);
                            ws.send(JSON.stringify({
                                action: 'selectRank',
                                args: { rank: action.rank, player: 'p1' }
                            }));
                        }
                        
                        // If we can select a player, select p2
                        const playerAction = actionKeys.find(k => k.includes('/players/'));
                        if (playerAction && ws.readyState === WebSocket.OPEN) {
                            const action = myActions[playerAction];
                            console.log(`[${playerName}] Selecting player:`, action.targetPlayer);
                            ws.send(JSON.stringify({
                                action: 'selectPlayer',
                                args: { targetPlayer: action.targetPlayer, player: 'p1' }
                            }));
                        }
                    }
                }
            }
        });
        
        ws.on('error', (error) => {
            console.error(`[${playerName}] Error:`, error);
            reject(error);
        });
    });
}

async function testGoFishLogs() {
    try {
        // Wait a bit for server to start
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        console.log('Creating Go Fish lobby...');
        const lobbyId = await createLobby();
        console.log(`Lobby created: ${lobbyId}`);
        
        console.log('\nConnecting players...');
        const alice = await connectPlayer(lobbyId, 'Alice');
        const bob = await connectPlayer(lobbyId, 'Bob');
        
        await new Promise(resolve => setTimeout(resolve, 500));
        
        console.log('\nStarting game...');
        alice.send(JSON.stringify({ action: 'start_game' }));
        
        // Let the game play out for a bit
        await new Promise(resolve => setTimeout(resolve, 10000));
        
        console.log('\nTest complete!');
        alice.close();
        bob.close();
        
    } catch (error) {
        console.error('Test failed:', error);
    }
}

testGoFishLogs();