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
                console.log(`[${playerName}] Action map:`, JSON.stringify(message.ui?.actionMap, null, 2));
            } else if (message.type === 'diff') {
                console.log(`[${playerName}] Diff received`);
                // Look for action map updates
                const actionMapPatch = message.patch?.find(p => p.path === '/ui/actionMap');
                if (actionMapPatch) {
                    console.log(`[${playerName}] Action map update:`, JSON.stringify(actionMapPatch.value, null, 2));
                }
                // Look for phase changes
                const phasePatch = message.patch?.find(p => p.path?.includes('phase'));
                if (phasePatch) {
                    console.log(`[${playerName}] Phase change:`, phasePatch);
                }
            }
        });
        
        ws.on('error', (error) => {
            console.error(`[${playerName}] Error:`, error);
            reject(error);
        });
    });
}

async function testGoFish() {
    try {
        console.log('Creating Go Fish lobby...');
        const lobbyId = await createLobby();
        console.log(`Lobby created: ${lobbyId}`);
        
        console.log('\nConnecting players...');
        const alice = await connectPlayer(lobbyId, 'Alice');
        const bob = await connectPlayer(lobbyId, 'Bob');
        
        await new Promise(resolve => setTimeout(resolve, 500));
        
        console.log('\nStarting game...');
        alice.send(JSON.stringify({ action: 'start_game' }));
        
        // Wait to see what happens
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        console.log('\nTest complete!');
        alice.close();
        bob.close();
        
    } catch (error) {
        console.error('Test failed:', error);
    }
}

testGoFish();