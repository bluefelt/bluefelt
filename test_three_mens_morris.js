#!/usr/bin/env node

const WebSocket = require('ws');

const LOBBY_ID = 'a454ab14d3';
const SERVER_URL = `ws://localhost:8000/api/lobbies/${LOBBY_ID}/ws`;

function createPlayer(playerName) {
    return new Promise((resolve, reject) => {
        const ws = new WebSocket(`${SERVER_URL}?player=${playerName}&join=true`);
        let connected = false;
        
        ws.on('open', () => {
            console.log(`${playerName} connected`);
            connected = true;
        });
        
        ws.on('message', (data) => {
            try {
                const message = JSON.parse(data.toString());
                console.log(`${playerName} received:`, JSON.stringify(message, null, 2));
                
                if (message.type === 'welcome') {
                    resolve(ws);
                }
            } catch (error) {
                console.error(`${playerName} parse error:`, error);
            }
        });
        
        ws.on('error', (error) => {
            console.error(`${playerName} error:`, error);
            if (!connected) reject(error);
        });
        
        ws.on('close', () => {
            console.log(`${playerName} disconnected`);
        });
    });
}

function sendAction(ws, playerName, action, args = {}) {
    const message = { action, args };
    console.log(`${playerName} sending:`, JSON.stringify(message));
    ws.send(JSON.stringify(message));
}

async function testThreeMensMorris() {
    try {
        console.log('=== Testing Three Men\'s Morris ===');
        
        // Connect players
        console.log('\n1. Connecting players...');
        const player1 = await createPlayer('Alice');
        const player2 = await createPlayer('Bob');
        
        // Wait a moment for connections to settle
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Start the game
        console.log('\n2. Starting game...');
        sendAction(player1, 'Alice', 'start_game');
        
        // Wait for game to start
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Test placement phase - each player places 3 pieces
        console.log('\n3. Testing placement phase...');
        
        // Alice places first piece at (0,0)
        sendAction(player1, 'Alice', 'placeToken', { target: '0-0', entity: 'piece_p1' });
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Bob places first piece at (1,1) 
        sendAction(player2, 'Bob', 'placeToken', { target: '1-1', entity: 'piece_p2' });
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Alice places second piece at (0,1)
        sendAction(player1, 'Alice', 'placeToken', { target: '0-1', entity: 'piece_p1' });
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Bob places second piece at (2,1)
        sendAction(player2, 'Bob', 'placeToken', { target: '2-1', entity: 'piece_p2' });
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Alice places third piece at (0,2) - should create a win condition (top row)
        sendAction(player1, 'Alice', 'placeToken', { target: '0-2', entity: 'piece_p1' });
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Bob places third piece at (2,2)
        sendAction(player2, 'Bob', 'placeToken', { target: '2-2', entity: 'piece_p2' });
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        console.log('\n4. Testing movement phase (if reached)...');
        
        // If we're in movement phase, test piece selection and movement
        // Alice selects her piece at (0,0)
        sendAction(player1, 'Alice', 'selectPiece', { location: '0-0', player: 'p1' });
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Alice moves selected piece to (1,0)
        sendAction(player1, 'Alice', 'moveSelectedPiece', { target: '1-0', player: 'p1' });
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        console.log('\n=== Test completed ===');
        
        // Clean up
        player1.close();
        player2.close();
        
    } catch (error) {
        console.error('Test failed:', error);
    }
}

testThreeMensMorris();