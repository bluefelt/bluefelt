const WebSocket = require('ws');

const lobbyId = 'ui_gPMukqO';
const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobbyId}/ws?player=TestPlayer&join=true`);

ws.on('open', () => {
    console.log('Connected to lobby');
    
    // Wait a bit then check lobby list
    setTimeout(() => {
        fetch('http://localhost:8000/api/lobbies')
            .then(res => res.json())
            .then(data => {
                console.log('\nLobby list:');
                console.log(JSON.stringify(data, null, 2));
                ws.close();
                process.exit(0);
            });
    }, 1000);
});

ws.on('message', (data) => {
    console.log('Received:', data.toString());
});

ws.on('error', (err) => {
    console.error('WebSocket error:', err);
    process.exit(1);
});
