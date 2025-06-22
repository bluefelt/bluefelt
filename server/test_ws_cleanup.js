#!/usr/bin/env node

const WebSocket = require('ws');

async function testWebSocketCleanup() {
    console.log('Testing WebSocket connection cleanup...\n');
    
    // Create lobby
    const response = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    
    const lobby = await response.json();
    console.log(`✓ Created lobby: ${lobby.id}`);
    
    // Test 1: Normal connection and disconnect
    console.log('\nTest 1: Normal connection and disconnect');
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=TestUser1&join=true`);
    
    await new Promise((resolve) => {
        ws1.on('open', () => {
            console.log('  ✓ Connected');
            setTimeout(() => {
                ws1.close();
                console.log('  ✓ Closed connection');
                resolve();
            }, 1000);
        });
    });
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Test 2: Abrupt disconnect (simulate network failure)
    console.log('\nTest 2: Abrupt disconnect');
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=TestUser2&join=true`);
    
    await new Promise((resolve) => {
        ws2.on('open', () => {
            console.log('  ✓ Connected');
            // Terminate connection abruptly
            ws2.terminate();
            console.log('  ✓ Terminated connection');
            resolve();
        });
    });
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Test 3: Multiple rapid connections
    console.log('\nTest 3: Multiple rapid connections');
    const connections = [];
    for (let i = 0; i < 5; i++) {
        const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=RapidUser${i}&join=true`);
        connections.push(ws);
        
        ws.on('open', () => {
            console.log(`  ✓ Connected RapidUser${i}`);
        });
        
        ws.on('error', (err) => {
            console.log(`  ✗ Error for RapidUser${i}: ${err.message}`);
        });
    }
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Close all connections
    connections.forEach((ws, i) => {
        ws.close();
        console.log(`  ✓ Closed RapidUser${i}`);
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Clean up
    await fetch(`http://localhost:8000/api/lobbies/${lobby.id}`, { method: 'DELETE' });
    console.log('\n✓ Cleanup completed');
}

testWebSocketCleanup().catch(console.error);