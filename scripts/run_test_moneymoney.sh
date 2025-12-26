#!/bin/bash
# Script to prepare and launch MoneyMoney test instance
# This ensures production data is not affected by integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_DATA_DIR="$PROJECT_ROOT/.test_data"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}MoneyMoney Test Instance Setup${NC}"
echo "================================"
echo ""

# Check if production MoneyMoney is running
PRODUCTION_RUNNING=$(pgrep -f "/Applications/MoneyMoney.app" || echo "")
if [ -n "$PRODUCTION_RUNNING" ]; then
    echo -e "${RED}ERROR: Production MoneyMoney is currently running!${NC}"
    echo ""
    echo "Due to MoneyMoney's database locking, you cannot run two instances simultaneously."
    echo ""
    echo "Options:"
    echo ""
    echo -e "  ${YELLOW}Option 1: Use production database for tests (RECOMMENDED)${NC}"
    echo "    - Tests use 'test-' prefixed accounts"
    echo "    - Tests don't modify your real accounts"
    echo "    - You can review and delete test accounts after"
    echo "    → Keep MoneyMoney running, create test accounts, run tests"
    echo ""
    echo -e "  ${YELLOW}Option 2: Use isolated test database${NC}"
    echo "    1. Quit production MoneyMoney"
    echo "    2. Run this script again to launch test instance"
    echo "    3. Create test accounts in test instance"
    echo "    4. Run tests"
    echo "    5. Quit test instance, restart production MoneyMoney"
    echo ""
    exit 1
fi

# Production MoneyMoney is not running, we can launch test instance
echo -e "${YELLOW}Setting up isolated test environment...${NC}"

# Get the container directory
PROD_CONTAINER="$HOME/Library/Containers/com.moneymoney-app.retail"
TEST_CONTAINER="$TEST_DATA_DIR/Containers/com.moneymoney-app.retail"

mkdir -p "$TEST_CONTAINER/Data/Library/Application Support/MoneyMoney"
mkdir -p "$TEST_CONTAINER/Data/Library/Preferences"

# Check if test database exists
DB_FILE="$TEST_CONTAINER/Data/Library/Application Support/MoneyMoney/Database.sql"
if [ -f "$DB_FILE" ]; then
    echo -e "${GREEN}✓${NC} Found existing test database"
    DB_SIZE=$(du -sh "$DB_FILE" | cut -f1)
    echo "  Location: $DB_FILE"
    echo "  Size: $DB_SIZE"
else
    echo -e "${GREEN}✓${NC} Will create fresh test database on first launch"
fi

echo ""
echo -e "${YELLOW}Backing up production container (safety measure)...${NC}"
BACKUP_DIR="$TEST_DATA_DIR/backups"
mkdir -p "$BACKUP_DIR"
BACKUP_NAME="container_backup_$(date +%Y%m%d_%H%M%S).tar.gz"
if [ -d "$PROD_CONTAINER" ]; then
    tar -czf "$BACKUP_DIR/$BACKUP_NAME" -C "$(dirname "$PROD_CONTAINER")" "$(basename "$PROD_CONTAINER")" 2>/dev/null || true
    echo -e "${GREEN}✓${NC} Production data backed up to: $BACKUP_DIR/$BACKUP_NAME"
fi

echo ""
echo -e "${YELLOW}Temporarily moving production container...${NC}"
if [ -d "$PROD_CONTAINER" ]; then
    mv "$PROD_CONTAINER" "$PROD_CONTAINER.backup"
    echo -e "${GREEN}✓${NC} Production container moved to: $PROD_CONTAINER.backup"
fi

echo -e "${YELLOW}Creating symlink to test container...${NC}"
ln -s "$TEST_CONTAINER" "$PROD_CONTAINER"
echo -e "${GREEN}✓${NC} Test container linked"

echo ""
echo -e "${BLUE}Launching MoneyMoney test instance...${NC}"
echo ""
open -a "MoneyMoney"

echo -e "${GREEN}✓${NC} MoneyMoney test instance launched!"
echo ""
echo -e "${YELLOW}IMPORTANT:${NC}"
echo "  1. Create test accounts in MoneyMoney:"
echo "     • test-cash (Cash Account, EUR)"
echo "     • test-checking (Giro Account, EUR)"
echo "  2. Run tests: cargo test --test roundtrip_tests -- --ignored"
echo "  3. When done, run: $SCRIPT_DIR/restore_production_moneymoney.sh"
echo ""
echo -e "${RED}Remember: Production MoneyMoney is temporarily disabled!${NC}"
echo "Run the restore script to get back to normal."
