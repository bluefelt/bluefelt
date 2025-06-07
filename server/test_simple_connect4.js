const WebSocket = require('ws');

async function test() {
    // Create lobby
    const res = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'connect-four' })
    });
    
    const lobby = await res.json();
    console.log('Lobby:', lobby.id);
    
    // Connect players
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=P1&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=P2&join=true`);
    
    ws1.on('message', (data) => {
        const msg = JSON.parse(data);
        if (msg.type === 'diff' && msg.patch) {
            console.log('\nP1 patches:');
            msg.patch.forEach(p => {
                if (p.path && p.path.includes('board')) {
                    console.log(`  ${p.op} ${p.path} = ${JSON.stringify(p.value)}`);
                }
            });
        }
    });
    
    // Wait and start
    await new Promise(r => setTimeout(r, 500));
    ws1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait and drop disc
    await new Promise(r => setTimeout(r, 500));
    console.log('\nDropping disc...');
    ws1.send(JSON.stringify({ action: 'dropDisc', targetColumn: 3 }));
    
    await new Promise(r => setTimeout(r, 1000));
    
    ws1.close();
    ws2.close();
}

test().catch(console.error);