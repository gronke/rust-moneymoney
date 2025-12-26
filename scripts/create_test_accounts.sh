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
if ! pgrep -x "MoneyMoney" > /dev/null; then
    echo -e "${YELLOW}Starting MoneyMoney...${NC}"
    open -a "MoneyMoney"
    sleep 3
fi

# Check if test accounts already exist
EXISTING=$(osascript -e 'tell application "MoneyMoney" to export accounts' 2>/dev/null | grep -c "test-cash\|test-checking" || true)
if [ "$EXISTING" -ge 2 ]; then
    echo -e "${GREEN}Test accounts already exist!${NC}"
    echo ""
    osascript -e 'tell application "MoneyMoney" to export accounts' | grep -B1 "test-" | grep "<string>" | sed 's/.*<string>\(.*\)<\/string>/  • \1/'
    echo ""
    echo "You can run integration tests:"
    echo "  cargo test --test roundtrip_tests -- --ignored --nocapture"
    exit 0
fi

echo -e "${YELLOW}Creating test accounts using UI automation...${NC}"
echo ""
echo "Note: This requires Accessibility permissions for Terminal/VSCode."
echo "If you see a permission error, grant access in System Settings:"
echo "  → Privacy & Security → Accessibility → Enable your terminal app"
echo ""

# Function to create a single account
create_account() {
    local account_name="$1"
    local account_type="$2"  # "cash" or "giro"

    osascript <<EOF
tell application "MoneyMoney"
    activate
end tell

delay 0.5

tell application "System Events"
    tell process "MoneyMoney"
        -- Open Account → Add Account...
        click menu item "Add Account..." of menu "Account" of menu bar 1
    end tell
end tell

delay 1

tell application "System Events"
    tell process "MoneyMoney"
        tell sheet 1 of window 1
            -- Click "Other" radio button
            click radio button 2
        end tell

        delay 0.5

        tell sheet 1 of window 1
            -- Click popup and type to select Offline account
            click pop up button 1
        end tell

        delay 0.5
        keystroke "off"
        delay 0.3
        keystroke return

        delay 0.5

        tell sheet 1 of window 1
            click button "Next"
        end tell
    end tell
end tell

delay 1

tell application "System Events"
    tell process "MoneyMoney"
        -- Tab to name field and type account name
        keystroke tab
        delay 0.3
        keystroke "a" using command down
        delay 0.2
        keystroke "${account_name}"

        delay 0.5

        tell sheet 1 of window 1
            -- Select account type
            click pop up button 1
        end tell

        delay 0.5
        keystroke "${account_type}"
        delay 0.3
        keystroke return

        delay 0.5

        tell sheet 1 of window 1
            click button "Done"
        end tell
    end tell
end tell

delay 1
EOF
}

# Create test-cash account
echo "Creating test-cash (Cash account)..."
if create_account "test-cash" "cash"; then
    echo -e "  ${GREEN}✓${NC} test-cash created"
else
    echo -e "  ${RED}✗${NC} Failed to create test-cash"
    exit 1
fi

sleep 1

# Create test-checking account
echo "Creating test-checking (Giro account)..."
if create_account "test-checking" "giro"; then
    echo -e "  ${GREEN}✓${NC} test-checking created"
else
    echo -e "  ${RED}✗${NC} Failed to create test-checking"
    exit 1
fi

# Verify accounts were created
echo ""
echo "Verifying accounts..."
sleep 1

ACCOUNTS=$(osascript -e 'tell application "MoneyMoney" to export accounts' 2>/dev/null)
HAS_CASH=$(echo "$ACCOUNTS" | grep -c "test-cash" || true)
HAS_CHECKING=$(echo "$ACCOUNTS" | grep -c "test-checking" || true)

if [ "$HAS_CASH" -ge 1 ] && [ "$HAS_CHECKING" -ge 1 ]; then
    echo -e "${GREEN}✅ Test accounts created successfully!${NC}"
    echo ""
    echo "Created accounts:"
    echo "  • test-cash (Cash Account, EUR)"
    echo "  • test-checking (Giro Account, EUR)"
    echo ""
    echo "You can now run integration tests:"
    echo "  cargo test --test roundtrip_tests -- --ignored --nocapture"
else
    echo -e "${RED}❌ Account verification failed${NC}"
    echo ""
    echo "The UI automation may need adjustment for your MoneyMoney version."
    echo "Please create the accounts manually:"
    echo "  1. Open MoneyMoney"
    echo "  2. Account → Add Account..."
    echo "  3. Select 'Other' → 'Offline account / Cash'"
    echo "  4. Click Next"
    echo "  5. Name: test-cash, Type: Cash account"
    echo "  6. Click Done"
    echo "  7. Repeat for test-checking (Type: Giro account)"
    exit 1
fi
