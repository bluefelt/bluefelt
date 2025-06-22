const WebSocket = require('ws');
const http = require('http');

// Make a simple HTTP request to test if server is responding
const options = {
  hostname: 'localhost',
  port: 8000,
  path: '/api/lobbies',
  method: 'POST',
  headers: { 'Content-Type': 'application/json' }
};

console.log('Testing server connection...');
const req = http.request(options, (res) => {
  console.log('Response status:', res.statusCode);
  let data = '';
  res.on('data', (chunk) => data += chunk);
  res.on('end', () => {
    try {
      const lobby = JSON.parse(data);
      console.log('Created lobby:', lobby.id);
      
      // Try to connect with WebSocket
      console.log('Connecting WebSocket...');
      const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=p1&join=true`);
      
      ws.on('open', () => {
        console.log('WebSocket connected!');
        ws.close();
        process.exit(0);
      });
      
      ws.on('error', (err) => {
        console.log('WebSocket error:', err.message);
        process.exit(1);
      });
      
      ws.on('close', () => {
        console.log('WebSocket closed');
      });
      
      setTimeout(() => {
        console.log('WebSocket connection timeout');
        ws.close();
        process.exit(1);
      }, 5000);
      
    } catch (e) {
      console.log('Failed to parse lobby response:', e.message);
      process.exit(1);
    }
  });
});

req.on('error', (err) => {
  console.log('HTTP request error:', err.message);
  process.exit(1);
});

req.write(JSON.stringify({game_id: 'tic-tac-toe'}));
req.end();