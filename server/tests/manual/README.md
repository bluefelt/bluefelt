# Manual Test Scripts

This directory contains manual test scripts used for debugging and testing specific scenarios.

## Directory Structure

- `js-scripts/` - JavaScript test scripts for WebSocket and integration testing
  - Connection tests
  - Game flow tests
  - Specific bug reproduction scripts

- `test-server-restart.sh` - Bash script for testing server restart scenarios

## Usage

### JavaScript Tests
These scripts are meant to be run with Node.js while the server is running:

```bash
# Start the server first
cd server && cargo run

# In another terminal, run a test script
cd server/tests/manual/js-scripts
node simple_lobby_test.js
```

### Shell Scripts
```bash
cd server/tests/manual
./test-server-restart.sh
```

## Important Notes

- These are manual test scripts, not automated tests
- They require the server to be running
- They may create test data (lobbies, games) that persist
- Use for debugging specific issues or reproducing bugs
- Not part of the regular test suite (cargo test)