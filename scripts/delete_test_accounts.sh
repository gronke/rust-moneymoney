#!/bin/bash
# Script to delete test accounts from MoneyMoney using UI automation
# Used to clean up after integration tests

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}MoneyMoney Test Account Cleanup${NC}"
echo "================================"
echo ""

# Check if MoneyMoney is running
if ! pgrep -x "MoneyMoney" > /dev/null; then
    echo -e "${YELLOW}Starting MoneyMoney...${NC}"
    open -a "MoneyMoney"
    sleep 3
fi

# Find test accounts
echo "Looking for test accounts..."
ACCOUNTS=$(osascript -e 'tell application "MoneyMoney" to export accounts' 2>/dev/null)

# Extract test account names (accounts starting with "test-")
TEST_ACCOUNTS=$(echo "$ACCOUNTS" | grep -o '<string>test-[^<]*</string>' | sed 's/<[^>]*>//g' | sort -u)

if [ -z "$TEST_ACCOUNTS" ]; then
    echo -e "${GREEN}No test accounts found. Environment is clean.${NC}"
    exit 0
fi

echo "Found test accounts:"
echo "$TEST_ACCOUNTS" | while read -r account; do
    echo "  • $account"
done
echo ""

# Function to delete an account by name
delete_account() {
    local account_name="$1"

    osascript <<EOF
tell application "MoneyMoney"
    activate
end tell

delay 0.1

tell application "System Events"
    tell process "MoneyMoney"
        -- Select the account in the sidebar
        tell outline 1 of scroll area 1 of splitter group 1 of window 1
            repeat with r in rows
                try
                    set txt to value of text field 1 of r
                    if txt contains "${account_name}" then
                        click r
                        delay 0.05
                        exit repeat
                    end if
                end try
            end repeat
        end tell
    end tell
end tell

delay 0.1

tell application "System Events"
    tell process "MoneyMoney"
        -- Open Remove Account menu
        click menu item "Remove Account..." of menu "Account" of menu bar 1
    end tell
end tell

delay 0.2

tell application "System Events"
    tell process "MoneyMoney"
        -- Click Delete Permanently in the confirmation dialog
        repeat with i from 1 to (count of windows)
            try
                click button "Delete Permanently" of window i
                exit repeat
            end try
        end repeat
    end tell
end tell

delay 0.1
EOF
}

# Delete each test account
echo -e "${YELLOW}Deleting test accounts...${NC}"
echo ""

echo "$TEST_ACCOUNTS" | while read -r account; do
    if [ -n "$account" ]; then
        echo -n "Deleting $account... "
        if delete_account "$account" 2>/dev/null; then
            echo -e "${GREEN}✓${NC}"
        else
            echo -e "${RED}✗${NC}"
        fi
        sleep 0.15
    fi
done

# Verify cleanup
echo ""
echo "Verifying cleanup..."
sleep 0.15

REMAINING=$(osascript -e 'tell application "MoneyMoney" to export accounts' 2>/dev/null | grep -c "test-" 2>/dev/null || true)
REMAINING=${REMAINING:-0}

if [ "$REMAINING" -eq 0 ]; then
    echo -e "${GREEN}✅ All test accounts removed successfully!${NC}"
    echo ""
    echo "Your MoneyMoney is now clean of test data."
else
    echo -e "${YELLOW}⚠️  Some test accounts may still exist${NC}"
    echo ""
    echo "Remaining test accounts:"
    osascript -e 'tell application "MoneyMoney" to export accounts' | grep -o '<string>test-[^<]*</string>' | sed 's/<[^>]*>//g' | while read -r account; do
        echo "  • $account"
    done
    echo ""
    echo "You may need to delete them manually:"
    echo "  1. Select the account in MoneyMoney sidebar"
    echo "  2. Account → Remove Account..."
    echo "  3. Click 'Delete Permanently'"
fi
