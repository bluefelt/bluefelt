#!/usr/bin/env node

// Comprehensive Go Fish logging test that simulates actual gameplay

const WebSocket = require('ws');

async function testGoFishLogging() {
  // Create lobby
  const createResponse = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'go-fish' })
  });
  
  const lobby = await createResponse.json();
  console.log('✓ Created lobby:', lobby.id);
  
  let logs = [];
  let gameState = null;
  let actionMap = null;
  
  return new Promise((resolve) => {
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    function processMessage(message, playerName) {
      if (message.type === 'stateUpdate') {
        // Update game state
        if (message.fullState) {
          gameState = message.fullState;
          actionMap = message.actionMap;
        }
        
        // Capture log messages
        if (message.patches) {
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
          
          // After game starts, simulate some actions
          setTimeout(() => {
            console.log('Alice selecting rank 4...');
            ws1.send(JSON.stringify({ 
              action: 'selectRank', 
              args: { player: 'p1', rank: '4' } 
            }));
            
            setTimeout(() => {
              console.log('Alice selecting Bob as target...');
              ws1.send(JSON.stringify({ 
                action: 'selectPlayer', 
                args: { player: 'p1', targetPlayer: 'p2' } 
              }));
            }, 1000);
          }, 2000);
        }, 500);
      }
      
      processMessage(message, 'Bob');
    });
    
    // Analyze results after enough time for actions
    setTimeout(() => {
      console.log('\n=== Go Fish Logging Analysis ===');
      console.log('Total log entries:', logs.length);
      
      // Test 1: Check for premature "will ask" logs (should be 0)
      const prematureLogs = logs.filter(log => log.message.includes('will ask for'));
      console.log('✅ Premature "will ask" logs:', prematureLogs.length, '(should be 0)');
      
      // Test 2: Check for "asks" logs
      const askLogs = logs.filter(log => log.message.includes(' asks ') && log.message.includes(' for '));
      console.log('✅ "Asks" logs:', askLogs.length, '(should be > 0)');
      
      // Test 3: Check for card transfer logs
      const transferLogs = logs.filter(log => log.message.includes('gives them to') || log.message.includes('has') && log.message.includes('and gives'));
      console.log('✅ Card transfer logs:', transferLogs.length);
      
      // Test 4: Check for "Go Fish" logs
      const goFishLogs = logs.filter(log => log.message.includes('Go Fish!'));
      console.log('✅ "Go Fish" logs:', goFishLogs.length);
      
      // Test 5: Check for pair formation logs
      const pairLogs = logs.filter(log => log.message.includes('forms a pair'));
      console.log('✅ Pair formation logs:', pairLogs.length);
      
      // Show all logs in order
      console.log('\n--- All Game Logs (in order) ---');
      logs.forEach((log, index) => {
        console.log(`${index + 1}. ${log.message}`);
      });
      
      // Check if our main fix worked: no premature "will ask" logs
      if (prematureLogs.length === 0) {
        console.log('\n✅ SUCCESS: No premature "will ask" logs found!');
      } else {
        console.log('\n❌ ISSUE: Found premature "will ask" logs');
      }
      
      // Check if we have proper asking logs
      if (askLogs.length > 0) {
        console.log('✅ SUCCESS: Found proper "asks" logs!');
      } else {
        console.log('❌ ISSUE: No "asks" logs found');
      }
      
      console.log('\n✅ Go Fish logging test completed');
      
      ws1.close();
      ws2.close();
      resolve();
    }, 15000);
  });
}

testGoFishLogging().then(() => {
  console.log('Test completed');
  process.exit(0);
}).catch(console.error);