const { TicTacToeRegressionTest } = require('./tests/regression/games/test-tic-tac-toe.js');

async function testTie() {
  const test = new TicTacToeRegressionTest();
  
  // Setup and start game
  await test.setupLobby();
  await test.connectPlayers();
  await test.startGame();
  
  console.log('\nInitial gameStatus:', test.gameStatus);
  
  // Play tie game moves
  const moves = [
    { player: 'p1', row: 0, col: 0 },
    { player: 'p2', row: 1, col: 1 },
    { player: 'p1', row: 0, col: 1 },
    { player: 'p2', row: 0, col: 2 },
    { player: 'p1', row: 2, col: 0 },
    { player: 'p2', row: 1, col: 0 },
    { player: 'p1', row: 1, col: 2 },
    { player: 'p2', row: 2, col: 2 },
    { player: 'p1', row: 2, col: 1 }
  ];
  
  for (const move of moves) {
    if (test.currentPlayer === move.player) {
      await test.executeAction(move.player, 'placeMark', {
        location: `/zones/board/cells/${move.row}/${move.col}`,
        entity: `mark_${move.player}`
      });
      console.log(`After ${move.player} move, gameStatus:`, test.gameStatus);
    }
  }
  
  console.log('\nFinal gameStatus:', test.gameStatus);
  console.log('Tie field exists:', test.gameStatus?.tie !== undefined);
  console.log('Tie value:', test.gameStatus?.tie);
  
  await test.cleanup();
}

testTie().catch(console.error);
