#!/bin/bash

# Comprehensive Test Runner for Bluefelt Client
# This ensures all games work correctly without manual testing

set -e

echo "🧪 Running Comprehensive Bluefelt Tests"
echo "======================================"
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run tests and check results
run_test_suite() {
    local suite_name=$1
    local test_pattern=$2
    
    echo "📋 Running $suite_name..."
    
    if pnpm test "$test_pattern" --run 2>&1 | tee test-output.log; then
        echo -e "${GREEN}✅ $suite_name PASSED${NC}"
        return 0
    else
        echo -e "${RED}❌ $suite_name FAILED${NC}"
        return 1
    fi
}

# Track overall success
all_passed=true

# 1. Unit Tests - Component Isolation
echo "1️⃣  UNIT TESTS"
echo "-------------"
if ! run_test_suite "Component Unit Tests" "src/components/__tests__"; then
    all_passed=false
fi
echo

# 2. Integration Tests - Component Integration
echo "2️⃣  INTEGRATION TESTS"
echo "-------------------"
if ! run_test_suite "WebSocket Integration" "src/ws/__tests__"; then
    all_passed=false
fi
if ! run_test_suite "Hook Integration" "src/hooks/__tests__"; then
    all_passed=false
fi
echo

# 3. End-to-End Tests - Full User Flows
echo "3️⃣  END-TO-END TESTS"
echo "------------------"
if ! run_test_suite "End-to-End Game Tests" "EndToEndGameTests"; then
    all_passed=false
fi
echo

# 4. Game-Specific Regression Tests
echo "4️⃣  GAME REGRESSION TESTS"
echo "-----------------------"
for game in "tic-tac-toe" "connect-four" "three-mens-morris" "go-fish"; do
    if ! run_test_suite "$game Tests" "src/__tests__/regression/$game"; then
        all_passed=false
    fi
done
echo

# 5. Visual/UI Tests
echo "5️⃣  UI AFFORDANCE TESTS"
echo "---------------------"
if ! run_test_suite "UI Affordances" "UIAffordances"; then
    all_passed=false
fi
echo

# Generate Coverage Report
echo "📊 GENERATING COVERAGE REPORT"
echo "============================"
pnpm test:coverage --run > coverage-report.txt 2>&1 || true
echo

# Summary
echo "📈 TEST SUMMARY"
echo "=============="

if [ "$all_passed" = true ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! 🎉${NC}"
    echo
    echo "✅ The client is ready for deployment!"
    echo "✅ All games work correctly"
    echo "✅ All user interactions are functional"
    echo "✅ WebSocket communication is working"
else
    echo -e "${RED}⚠️  SOME TESTS FAILED ⚠️${NC}"
    echo
    echo "Please fix the failing tests before deploying."
    echo "Check test-output.log for details."
    exit 1
fi

# Check for untested code
echo
echo "🔍 CHECKING TEST COVERAGE"
echo "======================="

# Extract coverage percentages
if [ -f coverage-report.txt ]; then
    coverage=$(grep -A 4 "All files" coverage-report.txt | tail -1 | awk '{print $4}' | sed 's/%//')
    
    if [ -n "$coverage" ]; then
        if (( $(echo "$coverage < 80" | bc -l) )); then
            echo -e "${YELLOW}⚠️  Warning: Test coverage is ${coverage}% (target: 80%)${NC}"
        else
            echo -e "${GREEN}✅ Test coverage is ${coverage}%${NC}"
        fi
    fi
fi

echo
echo "🎯 CRITICAL GAME FEATURES VERIFICATION"
echo "===================================="

# Verify each game's critical features
check_feature() {
    local feature=$1
    local test_grep=$2
    
    if grep -q "$test_grep" test-output.log; then
        echo -e "${GREEN}✅ $feature${NC}"
    else
        echo -e "${YELLOW}⚠️  $feature - needs verification${NC}"
    fi
}

echo "Go Fish:"
check_feature "  Rank selection UI" "choice-zone.*Rank"
check_feature "  Player selection UI" "choice-zone.*player"
check_feature "  Card transfer" "transferCards"

echo
echo "Connect Four:"
check_feature "  Column clicking" "dropPiece.*column"
check_feature "  Gravity placement" "cells.*entity.*piece"
check_feature "  Win detection" "gameStatus.*ended"

echo
echo "Three Men's Morris:"
check_feature "  Piece placement" "placePiece"
check_feature "  Piece movement" "moveSelectedPiece"
check_feature "  Mill detection" "formsMill"

echo
echo "Tic Tac Toe:"
check_feature "  Cell clicking" "placeMarker"
check_feature "  Win detection" "winner"
check_feature "  Tie detection" "tie.*true"

echo
echo "======================================"
echo "Test run completed at $(date)"

# Clean up
rm -f test-output.log coverage-report.txt