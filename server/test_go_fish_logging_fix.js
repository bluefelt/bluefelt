#!/usr/bin/env node

// Test Go Fish logging fixes:
// 1. No premature "will ask for" logs
// 2. Proper log ordering: "asks" before "Go Fish"

const WebSocket = require('ws');

async function testLoggingFix() {
  // Create lobby
  const createResponse = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'go-fish' })
  });
  
  const lobby = await createResponse.json();
  console.log('✓ Created lobby:', lobby.id);
  
  let logs = [];
  let gameStarted = false;
  
  return new Promise((resolve) => {
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    function processMessage(message, playerName) {
      if (message.type === 'stateUpdate' && message.patches) {
        // Capture log messages with timestamps for ordering analysis
        message.patches.forEach(patch => {
          if (patch.path === '/ui/gameLog/-' && patch.value?.message) {
            const logEntry = {
              message: patch.value.message,
              timestamp: new Date().toISOString(),
              player: playerName
            };
            logs.push(logEntry);
            console.log(`[${playerName}] Log: ${patch.value.message}`);
          }
        });
      }
    }
    
    ws1.on('message', (data) => {
      const message = JSON.parse(data);
      
      if (message.type === 'welcome') {
        console.log('✓ Alice connected');
      }
      
      processMessage(message, 'Alice');
    });
    
    ws2.on('message', (data) => {
      const message = JSON.parse(data);
      
      if (message.type === 'welcome') {
        console.log('✓ Bob connected');
        
        setTimeout(() => {
          console.log('Starting game...');
          ws1.send(JSON.stringify({ action: 'start_game' }));
          gameStarted = true;
        }, 500);
      }
      
      processMessage(message, 'Bob');
    });
    
    // Analyze results after game has time to progress
    setTimeout(() => {
      console.log('\n=== Logging Analysis ===');
      console.log('Total log entries:', logs.length);
      
      // Test 1: Check for premature "will ask" logs
      const prematureLogs = logs.filter(log => log.message.includes('will ask for'));
      console.log('✅ Premature "will ask" logs:', prematureLogs.length, '(should be 0)');
      
      // Test 2: Check log ordering (if we have both types)
      const askLogs = logs.filter(log => log.message.includes(' asks ') && log.message.includes(' for '));
      const goFishLogs = logs.filter(log => log.message.includes('Go Fish!'));
      
      console.log('✅ "Asks" logs:', askLogs.length);
      console.log('✅ "Go Fish" logs:', goFishLogs.length);
      
      // If we have both types, check ordering
      if (askLogs.length > 0 && goFishLogs.length > 0) {
        console.log('\n--- Log Order Analysis ---');
        logs.forEach((log, index) => {
          console.log(`${index + 1}. ${log.message}`);
        });
        
        // Find instances where "asks" and "Go Fish" appear in sequence
        for (let i = 0; i < logs.length - 1; i++) {
          const currentLog = logs[i];
          const nextLog = logs[i + 1];
          
          if (currentLog.message.includes(' asks ') && nextLog.message.includes('Go Fish!')) {
            console.log('✅ Correct order: "asks" before "Go Fish"');
          } else if (currentLog.message.includes('Go Fish!') && nextLog.message.includes(' asks ')) {
            console.log('❌ Wrong order: "Go Fish" before "asks"');
          }
        }
      }
      
      console.log('\n✅ Logging fix test completed');
      
      ws1.close();
      ws2.close();
      resolve();
    }, 10000);
  });
}

testLoggingFix().then(() => {
  console.log('Test completed');
  process.exit(0);
}).catch(console.error);