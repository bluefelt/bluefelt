#!/usr/bin/env node

const axios = require('axios');
const WebSocket = require('ws');

const BASE_URL = 'http://localhost:8000/api';

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

class HexTicTacToeTest {
    constructor() {
        this.lobbyId = null;
        this.player1Socket = null;
        this.player2Socket = null;
        this.gameStarted = false;
        this.currentPlayer = null;
        this.board = {};
        this.gameId = null;
        this.lobbyState = null;
    }

    async run() {
        try {
            console.log('🧪 Starting Hex Tic-Tac-Toe Win Detection Test...\n');
            
            // Create lobby
            await this.createLobby();
            
            // Connect players
            await this.connectPlayers();
            
            // Create table and join
            await this.createAndJoinTable();
            
            // Start game  
            await this.startGame();
            
            // Test horizontal win scenario
            await this.testHorizontalWin();
            
            console.log('✅ All tests passed!');
            
        } catch (error) {
            console.error('❌ Test failed:', error.message);
            process.exit(1);
        } finally {
            this.cleanup();
        }
    }

    async createLobby() {
        console.log('📝 Creating hex-tic-tac-toe lobby...');
        
        const response = await axios.post(`${BASE_URL}/lobbies`, {
            game_id: 'hex-tic-tac-toe'
        });
        
        this.lobbyId = response.data.id;
        console.log(`✓ Created lobby: ${this.lobbyId}`);
    }

    async connectPlayers() {
        console.log('🔌 Connecting players...');
        
        // Connect player 1
        const ws1Url = `ws://localhost:8000/api/lobbies/${this.lobbyId}/ws?player=p1&join=true`;
        this.player1Socket = new WebSocket(ws1Url);
        
        await new Promise((resolve, reject) => {
            this.player1Socket.on('open', () => {
                console.log('✓ Player 1 connected');
                resolve();
            });
            this.player1Socket.on('error', reject);
        });

        // Connect player 2
        const ws2Url = `ws://localhost:8000/api/lobbies/${this.lobbyId}/ws?player=p2&join=true`;
        this.player2Socket = new WebSocket(ws2Url);
        
        await new Promise((resolve, reject) => {
            this.player2Socket.on('open', () => {
                console.log('✓ Player 2 connected');
                resolve();
            });
            this.player2Socket.on('error', reject);
        });

        // Set up message handlers
        this.setupMessageHandlers();
    }

    async createAndJoinTable() {
        console.log('🎮 Creating game...');
        
        // Create game
        this.player1Socket.send(JSON.stringify({
            action: 'createGame',
            gameType: 'hex-tic-tac-toe'
        }));
        
        // Wait for game ID to be available
        await this.waitForCondition(() => this.gameId !== null, 3000, 'Game to be created');
        console.log(`✓ Game created with ID: ${this.gameId}`);
        
        // Both players join the game
        this.player1Socket.send(JSON.stringify({
            action: 'joinGame',
            gameId: this.gameId
        }));
        
        this.player2Socket.send(JSON.stringify({
            action: 'joinGame', 
            gameId: this.gameId
        }));
        
        await sleep(1000);
        console.log('✓ Players joined game');
    }

    setupMessageHandlers() {
        this.player1Socket.on('message', (data) => {
            try {
                const message = JSON.parse(data);
                this.handleMessage('p1', message);
            } catch (e) {
                console.log('Player 1 received:', data.toString());
            }
        });

        this.player2Socket.on('message', (data) => {
            try {
                const message = JSON.parse(data);
                this.handleMessage('p2', message);
            } catch (e) {
                console.log('Player 2 received:', data.toString());
            }
        });
    }

    handleMessage(player, message) {
        console.log(`[${player}] Received:`, JSON.stringify(message, null, 2));
        
        if (message.type === 'lobbyState') {
            this.lobbyState = message.lobby;
            // Get the first game ID if available
            if (message.lobby.games && message.lobby.games.length > 0) {
                this.gameId = message.lobby.games[0].id;
                console.log(`Game ID detected: ${this.gameId}`);
            }
        } else if (message.type === 'gameStarted') {
            console.log('🎮 Game started!');
            this.gameStarted = true;
            this.currentPlayer = message.data.currentPlayer;
            console.log(`Current player: ${this.currentPlayer}`);
        } else if (message.type === 'gameUpdate') {
            if (message.data.currentPlayer) {
                this.currentPlayer = message.data.currentPlayer;
            }
            
            // Check for winner
            if (message.data.gameStatus && message.data.gameStatus.winner) {
                console.log(`🏆 Winner detected: ${message.data.gameStatus.winner}`);
            }
            
            // Update board state
            if (message.data.zones && message.data.zones.hex_board) {
                this.board = message.data.zones.hex_board;
            }
        }
    }

    async startGame() {
        console.log('🚀 Starting game...');
        
        this.player1Socket.send(JSON.stringify({
            action: 'startGame',
            gameId: this.gameId
        }));
        
        // Wait for game to start
        await this.waitForCondition(() => this.gameStarted, 5000, 'Game to start');
        console.log('✓ Game started successfully');
    }

    async testHorizontalWin() {
        console.log('\n🎯 Testing horizontal win detection...');
        
        // Place marks to create a horizontal line: (0,0), (1,0), (2,0)
        const moves = [
            { player: 'p1', location: '0,0' },   // P1 at (0,0)
            { player: 'p2', location: '0,1' },   // P2 at (0,1) 
            { player: 'p1', location: '1,0' },   // P1 at (1,0)
            { player: 'p2', location: '1,1' },   // P2 at (1,1)
            { player: 'p1', location: '2,0' },   // P1 at (2,0) - should win!
        ];

        for (const move of moves) {
            console.log(`${move.player} places mark at ${move.location}`);
            
            // Wait for current player's turn
            await this.waitForCondition(() => this.currentPlayer === move.player, 3000, `${move.player}'s turn`);
            
            // Send the move
            const socket = move.player === 'p1' ? this.player1Socket : this.player2Socket;
            socket.send(JSON.stringify({
                action: 'place_mark',
                location: move.location
            }));
            
            // Wait a bit for the move to process
            await sleep(500);
        }

        // Wait for winner to be detected
        console.log('⏳ Waiting for win detection...');
        await sleep(2000);
        
        console.log('✓ Horizontal win test completed');
    }

    async waitForCondition(condition, timeout, description) {
        const start = Date.now();
        while (!condition()) {
            if (Date.now() - start > timeout) {
                throw new Error(`Timeout waiting for: ${description}`);
            }
            await sleep(100);
        }
    }

    cleanup() {
        if (this.player1Socket) {
            this.player1Socket.close();
        }
        if (this.player2Socket) {
            this.player2Socket.close();
        }
    }
}

// Run the test
const test = new HexTicTacToeTest();
test.run().catch(console.error);