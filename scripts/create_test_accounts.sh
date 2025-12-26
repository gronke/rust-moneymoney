#!/bin/bash
# Script to automatically create test accounts in MoneyMoney using UI automation
# This is a one-time setup for integration tests

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}MoneyMoney Test Account Setup${NC}"
echo "=============================="
echo ""

# Check if MoneyMoney is running
if ! pgrep -f "/Applications/MoneyMoney.app" > /dev/null; then
    echo -e "${YELLOW}Starting MoneyMoney...${NC}"
    open -a "MoneyMoney"
    sleep 3
fi

echo -e "${YELLOW}Creating test accounts using UI automation...${NC}"
echo ""
echo "Note: This requires Accessibility permissions for Terminal/iTerm."
echo "If you see a permission error, grant access in System Settings:"
echo "  → Privacy & Security → Accessibility → Enable Terminal"
echo ""

# AppleScript to create test-cash account
osascript <<'EOF'
tell application "MoneyMoney"
    activate
end tell

tell application "System Events"
    tell process "MoneyMoney"
        -- Wait for MoneyMoney to be ready
        delay 1

        -- Create first account: test-cash
        keystroke "n" using command down
        delay 1

        -- Navigate to Offline Account
        keystroke tab
        keystroke tab
        keystroke return
        delay 0.5

        -- Select Cash Account type
        keystroke return
        delay 0.5

        -- Enter account name
        keystroke "test-cash"
        keystroke tab

        -- Currency is already EUR by default
        keystroke tab
        keystroke tab

        -- Initial balance (0)
        keystroke "0"

        -- Click Create/OK button
        keystroke return
        delay 1

        -- Create second account: test-checking
        keystroke "n" using command down
        delay 1

        -- Navigate to Offline Account
        keystroke tab
        keystroke tab
        keystroke return
        delay 0.5

        -- Select Giro/Checking Account (down arrow once from Cash)
        keystroke (ASCII character 31) -- down arrow
        keystroke return
        delay 0.5

        -- Enter account name
        keystroke "test-checking"
        keystroke tab

        -- Currency is already EUR
        keystroke tab
        keystroke tab

        -- Initial balance (0)
        keystroke "0"

        -- Click Create/OK button
        keystroke return
        delay 1
    end tell
end tell
EOF

RESULT=$?

echo ""
if [ $RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ Test accounts created successfully!${NC}"
    echo ""
    echo "Created accounts:"
    echo "  • test-cash (Cash Account, EUR)"
    echo "  • test-checking (Giro Account, EUR)"
    echo ""
    echo "You can now run integration tests:"
    echo "  cargo test --test roundtrip_tests -- --ignored --nocapture"
else
    echo -e "${RED}❌ Failed to create test accounts${NC}"
    echo ""
    echo "The UI automation may need adjustment for your MoneyMoney version."
    echo "Please create the accounts manually (see test output for instructions)."
    exit 1
fi
