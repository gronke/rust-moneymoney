# Integration Test Setup Instructions

## One-Time Manual Setup Required

The roundtrip integration tests require two offline test accounts to be created manually in MoneyMoney. This is a one-time setup.

### Create Test Accounts

1. **Open MoneyMoney**

2. **Create First Test Account: test-cash**
   - Go to: File → New Account (⌘N)
   - Select: **Offline Account** → **Cash Account**
   - Name: `test-cash`
   - Currency: EUR
   - Initial Balance: 0.00
   - Click: Create

3. **Create Second Test Account: test-checking**
   - Go to: File → New Account (⌘N)
   - Select: **Offline Account** → **Giro Account** (Checking)
   - Name: `test-checking`
   - Currency: EUR
   - Initial Balance: 0.00
   - Click: Create

### Run Tests

Once the accounts are created, run:

```bash
cargo test --test roundtrip_tests -- --ignored --nocapture
```

### What the Tests Do

The tests will automatically:
1. ✅ Verify test accounts exist
2. ✅ Add 12 realistic transactions (from German merchants)
3. ✅ Export and verify transactions
4. ✅ Modify some transactions (add checkmarks, comments, categories)
5. ✅ Verify persistence

### Cleanup

**Currently: Tests do NOT clean up automatically**

After running tests, you can:
- Review the transactions in MoneyMoney
- Manually delete test accounts when done
- Or keep them for future test runs

### Future: Automated Cleanup

We'll add a cleanup function later that can:
- Delete all transactions from test accounts
- Optionally delete the test accounts themselves
- Reset MoneyMoney to pre-test state

For now, cleanup is manual to allow inspection of test results.
