#!/bin/bash
# Script to automatically create test accounts in MoneyMoney using UI automation.
# This is a one-time setup for integration tests.
#
# Each entry below is "name|type". The type must match a label MoneyMoney
# offers in the Add-Account dialog's account-type popup. Portfolio and
# Tagesgeld are deliberately omitted — those types are not creatable via
# the offline-account UI; they only appear on bank-synced accounts.

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

ACCOUNTS=(
    "test-cash|Cash account"
    "test-giro|Giro account"
    "test-savings|Savings account"
    "test-fixed-term|Fixed term deposit"
    "test-loan|Loan account"
    "test-creditcard|Credit card"
)

echo -e "${BLUE}MoneyMoney Test Account Setup${NC}"
echo "=============================="
echo ""

if ! pgrep -x "MoneyMoney" > /dev/null; then
    echo -e "${YELLOW}Starting MoneyMoney...${NC}"
    open -a "MoneyMoney"
    sleep 3
fi

EXISTING=$(osascript -e 'tell application "MoneyMoney" to export accounts' 2>/dev/null)

# Idempotency: skip accounts that already exist with the right type.
# Reads plist data on stdin; exit codes:
#   0 = account exists with the expected type
#   1 = account does not exist
#   2 = account exists but with a different type
account_exists_with_type() {
    local name="$1"
    local expected_type="$2"
    python3 -c '
import plistlib, sys
name, expected = sys.argv[1], sys.argv[2]
data = sys.stdin.buffer.read()
accs = plistlib.loads(data) if data else []
for a in accs:
    if a.get("name") == name:
        sys.exit(0 if a.get("type") == expected else 2)
sys.exit(1)
' "$name" "$expected_type"
}

create_account() {
    local name="$1"
    local type_label="$2"

    osascript <<EOF
tell application "MoneyMoney" to activate
delay 0.3
tell application "System Events"
    tell process "MoneyMoney"
        click menu item "Add Account..." of menu "Account" of menu bar 1
    end tell
end tell
delay 0.6
tell application "System Events"
    tell process "MoneyMoney"
        tell sheet 1 of window 1
            -- "Other" radio button (offline accounts and bank-specific cards)
            click radio button 2
        end tell
        delay 0.2
        tell sheet 1 of window 1
            click pop up button 1
        end tell
        delay 0.3
        keystroke "off"
        delay 0.2
        keystroke return
        delay 0.3
        tell sheet 1 of window 1
            click button "Next"
        end tell
    end tell
end tell
delay 0.6
tell application "System Events"
    tell process "MoneyMoney"
        tell sheet 1 of window 1
            -- text field 7 is "Icon & Name"; pop up button 1 is the type selector.
            set value of text field 7 to "${name}"
            delay 0.3
            click pop up button 1
            delay 0.5
            click menu item "${type_label}" of menu 1 of pop up button 1
            delay 0.4
            click button "Done"
        end tell
    end tell
end tell
delay 0.7
EOF
}

CREATED=0
SKIPPED=0
FAILED=0

for entry in "${ACCOUNTS[@]}"; do
    name="${entry%%|*}"
    type_label="${entry##*|}"

    # Re-export each iteration so previously-created accounts are visible.
    EXISTING=$(osascript -e 'tell application "MoneyMoney" to export accounts' 2>/dev/null)

    set +e
    echo "$EXISTING" | account_exists_with_type "$name" "$type_label"
    status=$?
    set -e

    case $status in
        0)
            echo -e "  ${GREEN}[skip]${NC} $name already exists ($type_label)"
            SKIPPED=$((SKIPPED + 1))
            continue
            ;;
        2)
            echo -e "  ${YELLOW}[WARN]${NC} $name exists with wrong type. Delete it first via scripts/delete_test_accounts.sh"
            FAILED=$((FAILED + 1))
            continue
            ;;
    esac

    echo -n "Creating $name ($type_label)... "
    if create_account "$name" "$type_label" 2>/dev/null; then
        echo -e "${GREEN}[ok]${NC}"
        CREATED=$((CREATED + 1))
    else
        echo -e "${RED}[FAIL]${NC}"
        FAILED=$((FAILED + 1))
    fi
    sleep 0.2
done

echo ""
echo "Verifying accounts..."
sleep 0.3
FINAL=$(osascript -e 'tell application "MoneyMoney" to export accounts' 2>/dev/null)
ALL_GOOD=1
for entry in "${ACCOUNTS[@]}"; do
    name="${entry%%|*}"
    type_label="${entry##*|}"
    set +e
    echo "$FINAL" | account_exists_with_type "$name" "$type_label"
    status=$?
    set -e
    if [ $status -eq 0 ]; then
        echo -e "  ${GREEN}[ok]${NC} $name ($type_label)"
    else
        echo -e "  ${RED}[FAIL]${NC} $name ($type_label) -- verification failed (status $status)"
        ALL_GOOD=0
    fi
done

echo ""
echo "Summary: ${CREATED} created, ${SKIPPED} skipped, ${FAILED} failed"

if [ $ALL_GOOD -eq 1 ]; then
    echo -e "${GREEN}OK: All test accounts ready.${NC}"
    echo ""
    echo "Run the integration tests:"
    echo "  cargo test --features test-utils --test integration_tests -- --ignored --nocapture"
else
    echo -e "${RED}ERROR: Some accounts failed verification.${NC}"
    echo ""
    echo "If UI automation broke, you can create accounts manually:"
    echo "  1. MoneyMoney -> Account -> Add Account..."
    echo "  2. Other -> Offline account / Cash -> Next"
    echo "  3. Set name and type from the dropdown"
    echo "  4. Click Done"
    exit 1
fi
