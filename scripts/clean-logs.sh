#!/bin/bash
# Clean up old log files from the Bluefelt project

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Bluefelt Log Cleanup Script${NC}"
echo "============================="

# Check if logs directory exists
if [ ! -d "logs" ]; then
    echo -e "${YELLOW}No logs directory found. Nothing to clean.${NC}"
    exit 0
fi

# Function to clean logs older than N days
clean_old_logs() {
    local dir=$1
    local days=$2
    local count=$(find "$dir" -name "*.log" -o -name "*.txt" -mtime +$days 2>/dev/null | wc -l)
    
    if [ $count -gt 0 ]; then
        echo -e "${YELLOW}Found $count log files older than $days days in $dir${NC}"
        find "$dir" -name "*.log" -o -name "*.txt" -mtime +$days -exec rm {} \;
        echo -e "${GREEN}Cleaned up $count old log files${NC}"
    else
        echo "No log files older than $days days in $dir"
    fi
}

# Clean logs older than 7 days by default
DAYS=${1:-7}

echo "Cleaning log files older than $DAYS days..."
echo

# Clean each subdirectory
for subdir in logs/server logs/client logs/tests logs/build; do
    if [ -d "$subdir" ]; then
        clean_old_logs "$subdir" "$DAYS"
    fi
done

echo
echo -e "${GREEN}Log cleanup complete!${NC}"

# Show current disk usage
echo
echo "Current log directory sizes:"
du -sh logs/* 2>/dev/null | sort -h

echo
echo "Tip: Run this script regularly to keep log files under control"
echo "Usage: ./scripts/clean-logs.sh [days]  (default: 7 days)"