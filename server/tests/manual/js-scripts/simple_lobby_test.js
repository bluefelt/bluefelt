const fetch = require('node-fetch');

async function simpleTest() {
  console.log('Testing basic lobby API...');
  
  try {
    console.log('1. Fetching initial lobbies...');
    const response = await Promise.race([
      fetch('http://localhost:8000/api/lobbies'),
      new Promise((_, reject) => setTimeout(() => reject(new Error('Timeout after 5s')), 5000))
    ]);
    
    if (response.ok) {
      const data = await response.json();
      console.log('✅ Success! Found lobbies:', data.length);
      return data;
    } else {
      console.log('❌ Bad response:', response.status);
    }
  } catch (error) {
    console.log('❌ Error:', error.message);
  }
}

simpleTest().then(() => process.exit(0));