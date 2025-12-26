#!/bin/bash
# Script to restore production MoneyMoney after testing
# Reverses the changes made by run_test_moneymoney.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}Restoring Production MoneyMoney${NC}"
echo "================================"
echo ""

# Check if MoneyMoney is running
MM_RUNNING=$(pgrep -f "/Applications/MoneyMoney.app" || echo "")
if [ -n "$MM_RUNNING" ]; then
    echo -e "${YELLOW}Quitting MoneyMoney test instance...${NC}"
    osascript -e 'quit app "MoneyMoney"' 2>/dev/null || true
    sleep 2
    echo -e "${GREEN}✓${NC} MoneyMoney quit"
fi

PROD_CONTAINER="$HOME/Library/Containers/com.moneymoney-app.retail"

# Check if we're in test mode
if [ ! -L "$PROD_CONTAINER" ]; then
    echo -e "${YELLOW}Note: Production container is not a symlink${NC}"
    echo "Either:"
    echo "  - Tests were not run with isolated database"
    echo "  - Already restored to production"
    echo ""
    if [ -d "$PROD_CONTAINER.backup" ]; then
        echo -e "${RED}WARNING: Found backup but current is not a symlink!${NC}"
        echo "Manual intervention may be needed."
        echo ""
        echo "Backup location: $PROD_CONTAINER.backup"
    fi
    exit 0
fi

echo -e "${YELLOW}Removing test symlink...${NC}"
rm "$PROD_CONTAINER"
echo -e "${GREEN}✓${NC} Test symlink removed"

echo -e "${YELLOW}Restoring production container...${NC}"
if [ -d "$PROD_CONTAINER.backup" ]; then
    mv "$PROD_CONTAINER.backup" "$PROD_CONTAINER"
    echo -e "${GREEN}✓${NC} Production container restored"
else
    echo -e "${RED}ERROR: Production backup not found!${NC}"
    echo ""
    echo "Expected location: $PROD_CONTAINER.backup"
    echo ""
    echo "You may need to restore from the backup archive in:"
    echo "  $PROJECT_ROOT/.test_data/backups/"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ Production MoneyMoney restored successfully!${NC}"
echo ""
echo "You can now launch MoneyMoney normally."
echo ""
echo "Test data remains at: $PROJECT_ROOT/.test_data/"
echo "To clean up: rm -rf $PROJECT_ROOT/.test_data/"
