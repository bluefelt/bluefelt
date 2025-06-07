const WebSocket = require('ws');

async function testPairLogging() {
    console.log('Testing pair formation logging in Go Fish...');
    
    // Create lobby
    const resp = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'go-fish' })
    });
    const lobby = await resp.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect two players
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    let gameLog = [];
    
    // Wait for connections
    await new Promise(resolve => {
        let connected = 0;
        ws1.on('open', () => { connected++; if (connected === 2) resolve(); });
        ws2.on('open', () => { connected++; if (connected === 2) resolve(); });
    });
    
    console.log('Both players connected');
    
    // Listen for patches that include game log updates
    const logPromise = new Promise(resolve => {
        const handleMessage = (data) => {
            try {
                const msg = JSON.parse(data);
                if (msg.type === 'patches' && msg.data) {
                    for (const patch of msg.data) {
                        if (patch.path && patch.path.includes('/ui/gameLog') && patch.value) {
                            gameLog.push(patch.value.message || patch.value);
                            console.log('LOG:', patch.value.message || patch.value);
                            
                            // Check for pair formation message
                            if (patch.value.message && patch.value.message.includes('forms a pair')) {
                                resolve(patch.value.message);
                            }
                        }
                    }
                }
            } catch (e) {
                // Ignore parse errors
            }
        };
        
        ws1.on('message', handleMessage);
        ws2.on('message', handleMessage);
    });
    
    // Start game
    console.log('Starting game...');
    ws1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for pair formation log or timeout
    const result = await Promise.race([
        logPromise,
        new Promise(resolve => setTimeout(() => resolve('TIMEOUT'), 10000))
    ]);
    
    if (result === 'TIMEOUT') {
        console.log('❌ TIMEOUT: No pair formation logged within 10 seconds');
        console.log('Game log entries received:', gameLog.length);
        console.log('Latest logs:', gameLog.slice(-5));
    } else {
        console.log('✅ SUCCESS: Pair formation logged:', result);
    }
    
    ws1.close();
    ws2.close();
}

testPairLogging().catch(console.error);