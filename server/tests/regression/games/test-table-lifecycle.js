const WebSocket = require('ws');
const { GameTestFramework } = require('../framework/GameTestFramework');

/**
 * Tests for the complete table lifecycle: creation → seating → ready → countdown → game start
 */
class TableLifecycleTest extends GameTestFramework {
    constructor() {
        super('Table Lifecycle Test');
        this.serverUrl = 'http://localhost:8000';
        this.wsUrl = 'ws://localhost:8000';
    }

    async testTableCreationAndSeating() {
        console.log('\n=== Testing Table Creation and Seating ===');
        
        // Create lobby
        const lobbyResponse = await fetch(`${this.serverUrl}/api/lobbies`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ game_id: 'tic-tac-toe' })
        });
        
        const lobby = await lobbyResponse.json();
        console.log('Created lobby:', lobby.id);
        
        // Connect two players
        const ws1 = new WebSocket(`${this.wsUrl}/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
        const ws2 = new WebSocket(`${this.wsUrl}/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
        
        await this.waitForConnection(ws1);
        await this.waitForConnection(ws2);
        
        // Player 1 creates a table
        const tableCreated = await this.sendAndWaitForResponse(ws1, {
            action: 'createTable',
            bundleId: 'tic-tac-toe',
            name: 'Test Table'
        }, 'tableCreated');
        
        console.log('Table created:', tableCreated.table.id);
        this.assertEqual(tableCreated.table.owner, 'Alice', 'Table owner should be Alice');
        
        // Both players should receive the notification
        const ws2TableNotification = await this.waitForMessage(ws2, msg => msg.type === 'tableCreated');
        this.assertEqual(ws2TableNotification.table.id, tableCreated.table.id, 'Both players should see same table');
        
        // Player 1 claims seat 0
        const seat1Claimed = await this.sendAndWaitForResponse(ws1, {
            action: 'claimSeat',
            tableId: tableCreated.table.id,
            seatIndex: 0
        }, 'seatClaimed');
        
        this.assertEqual(seat1Claimed.playerId, 'Alice', 'Alice should claim seat 0');
        
        // Player 2 claims seat 1
        const seat2Claimed = await this.sendAndWaitForResponse(ws2, {
            action: 'claimSeat',
            tableId: tableCreated.table.id,
            seatIndex: 1
        }, 'seatClaimed');
        
        this.assertEqual(seat2Claimed.playerId, 'Bob', 'Bob should claim seat 1');
        
        // Cleanup
        ws1.close();
        ws2.close();
        
        return true;
    }

    async testReadyAndCountdown() {
        console.log('\n=== Testing Ready States and Countdown ===');
        
        // Create lobby and table with players seated
        const { lobby, tableId, ws1, ws2 } = await this.setupTableWithPlayers();
        
        // Player 1 marks ready
        const ready1 = await this.sendAndWaitForResponse(ws1, {
            action: 'setReady',
            tableId: tableId,
            ready: true
        }, 'readyStateChanged');
        
        this.assertEqual(ready1.playerId, 'Alice', 'Ready state should be for Alice');
        this.assertEqual(ready1.ready, true, 'Alice should be ready');
        
        // Player 2 should also receive the notification
        const ready1Notification = await this.waitForMessage(ws2, msg => 
            msg.type === 'readyStateChanged' && msg.playerId === 'Alice'
        );
        this.assertEqual(ready1Notification.ready, true, 'Bob should see Alice is ready');
        
        // Player 2 marks ready - should trigger countdown
        ws2.send(JSON.stringify({
            action: 'setReady',
            tableId: tableId,
            ready: true
        }));
        
        // Both players should receive ready state change
        const ready2 = await this.waitForMessage(ws1, msg => 
            msg.type === 'readyStateChanged' && msg.playerId === 'Bob'
        );
        this.assertEqual(ready2.ready, true, 'Alice should see Bob is ready');
        
        // Both players should receive countdown started
        const countdown1 = await this.waitForMessage(ws1, msg => msg.type === 'countdownStarted');
        const countdown2 = await this.waitForMessage(ws2, msg => msg.type === 'countdownStarted');
        
        this.assertExists(countdown1.endsAt, 'Countdown should have end time');
        this.assertEqual(countdown1.tableId, tableId, 'Countdown should be for correct table');
        
        // Cleanup
        ws1.close();
        ws2.close();
        
        return true;
    }

    async testCountdownToGameStart() {
        console.log('\n=== Testing Countdown to Game Start ===');
        
        // Create lobby and table with players ready
        const { lobby, tableId, ws1, ws2 } = await this.setupReadyTable();
        
        // Wait for countdown to complete (using shorter test countdown)
        console.log('Waiting for countdown to complete...');
        
        // Both players should receive game started message
        const gameStarted1 = await this.waitForMessage(ws1, msg => msg.type === 'gameStarted', 3000);
        const gameStarted2 = await this.waitForMessage(ws2, msg => msg.type === 'gameStarted', 3000);
        
        this.assertExists(gameStarted1.gameState, 'Game state should be included');
        this.assertEqual(gameStarted1.tableId, tableId, 'Game should start for correct table');
        
        // Verify table status is now Playing
        const tablesResponse = await fetch(`${this.serverUrl}/api/lobbies/${lobby.id}/tables`);
        const tables = await tablesResponse.json();
        const table = tables.find(t => t.id === tableId);
        
        this.assertEqual(table.status, 'Playing', 'Table status should be Playing');
        
        // Cleanup
        ws1.close();
        ws2.close();
        
        return true;
    }

    async testChatIntegration() {
        console.log('\n=== Testing Chat Integration ===');
        
        // Create lobby
        const lobbyResponse = await fetch(`${this.serverUrl}/api/lobbies`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ game_id: 'tic-tac-toe' })
        });
        
        const lobby = await lobbyResponse.json();
        
        // Connect two players
        const ws1 = new WebSocket(`${this.wsUrl}/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
        const ws2 = new WebSocket(`${this.wsUrl}/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
        
        await this.waitForConnection(ws1);
        await this.waitForConnection(ws2);
        
        // Send lobby chat message
        const lobbyMsg = await this.sendAndWaitForResponse(ws1, {
            action: 'sendChatMessage',
            message: 'Hello lobby!',
            scope: 'lobby'
        }, 'chatMessage');
        
        this.assertEqual(lobbyMsg.scope, 'lobby', 'Should be lobby message');
        this.assertEqual(lobbyMsg.message.content, 'Hello lobby!', 'Message content should match');
        
        // Player 2 should also receive it
        const lobbyMsg2 = await this.waitForMessage(ws2, msg => 
            msg.type === 'chatMessage' && msg.scope === 'lobby'
        );
        this.assertEqual(lobbyMsg2.message.content, 'Hello lobby!', 'Bob should see lobby message');
        
        // Create table for table chat
        const tableCreated = await this.sendAndWaitForResponse(ws1, {
            action: 'createTable',
            bundleId: 'tic-tac-toe'
        }, 'tableCreated');
        
        // Clear Bob's notification
        await this.waitForMessage(ws2, msg => msg.type === 'tableCreated');
        
        // Send table chat message
        const tableMsg = await this.sendAndWaitForResponse(ws1, {
            action: 'sendChatMessage',
            message: 'Hello table!',
            scope: 'table',
            tableId: tableCreated.table.id
        }, 'chatMessage');
        
        this.assertEqual(tableMsg.scope, 'table', 'Should be table message');
        this.assertEqual(tableMsg.tableId, tableCreated.table.id, 'Should have table ID');
        
        // Cleanup
        ws1.close();
        ws2.close();
        
        return true;
    }

    async testSpectatorFlow() {
        console.log('\n=== Testing Spectator Flow ===');
        
        // Create lobby and table with 2 seats filled
        const { lobby, tableId, ws1, ws2 } = await this.setupTableWithPlayers();
        
        // Connect third player as spectator
        const ws3 = new WebSocket(`${this.wsUrl}/api/lobbies/${lobby.id}/ws?player=Charlie&join=true`);
        await this.waitForConnection(ws3);
        
        // Charlie should still receive table updates even as spectator
        ws1.send(JSON.stringify({
            action: 'setReady',
            tableId: tableId,
            ready: true
        }));
        
        const readyNotification = await this.waitForMessage(ws3, msg => 
            msg.type === 'readyStateChanged'
        );
        this.assertEqual(readyNotification.playerId, 'Alice', 'Spectator should see ready state changes');
        
        // Bob leaves his seat
        const seatReleased = await this.sendAndWaitForResponse(ws2, {
            action: 'releaseSeat',
            tableId: tableId,
            seatIndex: 1
        }, 'seatReleased');
        
        // Charlie can now claim the empty seat
        const charlieSeat = await this.sendAndWaitForResponse(ws3, {
            action: 'claimSeat',
            tableId: tableId,
            seatIndex: 1
        }, 'seatClaimed');
        
        this.assertEqual(charlieSeat.playerId, 'Charlie', 'Charlie should claim the seat');
        
        // Cleanup
        ws1.close();
        ws2.close();
        ws3.close();
        
        return true;
    }

    // Helper methods
    async setupTableWithPlayers() {
        // Create lobby
        const lobbyResponse = await fetch(`${this.serverUrl}/api/lobbies`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ game_id: 'tic-tac-toe' })
        });
        
        const lobby = await lobbyResponse.json();
        
        // Connect players
        const ws1 = new WebSocket(`${this.wsUrl}/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
        const ws2 = new WebSocket(`${this.wsUrl}/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
        
        await this.waitForConnection(ws1);
        await this.waitForConnection(ws2);
        
        // Create table
        const tableCreated = await this.sendAndWaitForResponse(ws1, {
            action: 'createTable',
            bundleId: 'tic-tac-toe'
        }, 'tableCreated');
        
        // Clear Bob's notification
        await this.waitForMessage(ws2, msg => msg.type === 'tableCreated');
        
        // Both players claim seats
        await this.sendAndWaitForResponse(ws1, {
            action: 'claimSeat',
            tableId: tableCreated.table.id,
            seatIndex: 0
        }, 'seatClaimed');
        
        // Clear Bob's notification
        await this.waitForMessage(ws2, msg => msg.type === 'seatClaimed');
        
        await this.sendAndWaitForResponse(ws2, {
            action: 'claimSeat',
            tableId: tableCreated.table.id,
            seatIndex: 1
        }, 'seatClaimed');
        
        // Clear Alice's notification
        await this.waitForMessage(ws1, msg => msg.type === 'seatClaimed' && msg.playerId === 'Bob');
        
        return { lobby, tableId: tableCreated.table.id, ws1, ws2 };
    }

    async setupReadyTable() {
        const { lobby, tableId, ws1, ws2 } = await this.setupTableWithPlayers();
        
        // Both players mark ready
        ws1.send(JSON.stringify({
            action: 'setReady',
            tableId: tableId,
            ready: true
        }));
        
        // Wait for notifications
        await this.waitForMessage(ws2, msg => msg.type === 'readyStateChanged');
        
        ws2.send(JSON.stringify({
            action: 'setReady',
            tableId: tableId,
            ready: true
        }));
        
        // Wait for countdown to start
        await this.waitForMessage(ws1, msg => msg.type === 'countdownStarted');
        await this.waitForMessage(ws2, msg => msg.type === 'countdownStarted');
        
        return { lobby, tableId, ws1, ws2 };
    }

    async runAllTests() {
        const tests = [
            () => this.testTableCreationAndSeating(),
            () => this.testReadyAndCountdown(),
            () => this.testCountdownToGameStart(),
            () => this.testChatIntegration(),
            () => this.testSpectatorFlow(),
        ];

        let passed = 0;
        let failed = 0;

        for (const test of tests) {
            try {
                await test();
                passed++;
                console.log('✓ Test passed');
            } catch (error) {
                failed++;
                console.error('✗ Test failed:', error.message);
                console.error(error.stack);
            }
        }

        console.log(`\n=== Table Lifecycle Test Results ===`);
        console.log(`Passed: ${passed}`);
        console.log(`Failed: ${failed}`);
        console.log(`Total: ${tests.length}`);

        return failed === 0;
    }
}

// Run tests if called directly
if (require.main === module) {
    const test = new TableLifecycleTest();
    test.runAllTests().then(success => {
        process.exit(success ? 0 : 1);
    });
}

module.exports = { TableLifecycleTest };