# End-to-end test for Plurcast with encrypted credentials
# Tests: credential storage, posting to Nostr, history verification

$ErrorActionPreference = "Continue"

# Test configuration
$MASTER_PASSWORD = "test_password_12345"
$TEST_KEY_HEX = "9270ffc3ddd551bf37a1417d5b0762a9f0a75204a3d6839c5d7e8790b1f57cad"
$TEST_NPUB = "npub1ch642h2jvaq2fv3pzq36m5t99nrzvppkdr6pw8m8eryfzezynzlqky6cjp"

Write-Host "`n=== Plurcast End-to-End Test ===" -ForegroundColor Cyan
Write-Host "Version:" -NoNewline; & .\target\release\plur-post.exe --version
Write-Host ""

# Step 1: Set environment variable for master password
Write-Host "[1/6] Setting master password..." -ForegroundColor Yellow
$env:PLURCAST_MASTER_PASSWORD = $MASTER_PASSWORD
Write-Host "  [OK] Master password set" -ForegroundColor Green

# Step 2: Store Nostr credentials via stdin
Write-Host "`n[2/6] Storing Nostr credentials..." -ForegroundColor Yellow
$storeOutput = echo $TEST_KEY_HEX | & .\target\release\plur-creds.exe set nostr --stdin 2>&1 | Out-String
$storeExitCode = $LASTEXITCODE

if ($storeExitCode -eq 0) {
    Write-Host "  [OK] Credentials stored" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Failed to store credentials (exit code: $storeExitCode)" -ForegroundColor Red
    Write-Host $storeOutput
    exit 1
}

# Step 3: Test credentials
Write-Host "`n[3/6] Testing Nostr credentials..." -ForegroundColor Yellow
$testResult = & .\target\release\plur-creds.exe test nostr 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  [OK] Credentials verified" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Credential test failed" -ForegroundColor Red
    Write-Host $testResult
    exit 1
}

# Step 4: Post to Nostr
Write-Host "`n[4/6] Posting test message to Nostr..." -ForegroundColor Yellow
$testMessage = "E2E test from Plurcast 0.3.0-alpha2 - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
$postResult = echo $testMessage | & .\target\release\plur-post.exe --platform nostr --format json 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "  [OK] Post successful" -ForegroundColor Green
    Write-Host "  Response: $postResult" -ForegroundColor Gray
} else {
    Write-Host "  [FAIL] Post failed" -ForegroundColor Red
    Write-Host $postResult
    exit 1
}

# Step 5: Verify in history
Write-Host "`n[5/6] Checking history..." -ForegroundColor Yellow
$history = & .\target\release\plur-history.exe --limit 1 --format json 2>&1
$historyObj = $history | ConvertFrom-Json

if ($historyObj -and $historyObj[0].platforms.Count -gt 0) {
    $nostrPost = $historyObj[0].platforms | Where-Object { $_.platform -eq "nostr" }
    if ($nostrPost -and $nostrPost.success) {
        Write-Host "  [OK] Post found in history" -ForegroundColor Green
        Write-Host "    Post ID: $($historyObj[0].post_id)" -ForegroundColor Gray
        Write-Host "    Nostr ID: $($nostrPost.platform_post_id)" -ForegroundColor Gray
        Write-Host "    Content: $($historyObj[0].content.Substring(0, [Math]::Min(50, $historyObj[0].content.Length)))..." -ForegroundColor Gray
    } else {
        Write-Host "  [FAIL] Nostr post not successful in history" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "  [FAIL] No platforms in history" -ForegroundColor Red
    exit 1
}

# Step 6: Re-test credentials (persistence check)
Write-Host "`n[6/6] Re-testing credentials (persistence check)..." -ForegroundColor Yellow
$testResult2 = & .\target\release\plur-creds.exe test nostr 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  [OK] Credentials still valid (encrypted storage persists)" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] Credentials lost (persistence failure)" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
Write-Host "[OK] All checks passed" -ForegroundColor Green
Write-Host "  - Encrypted credential storage: WORKING" -ForegroundColor Green
Write-Host "  - Nostr posting: WORKING" -ForegroundColor Green
Write-Host "  - History tracking: WORKING" -ForegroundColor Green
Write-Host "  - Credential persistence: WORKING" -ForegroundColor Green
Write-Host ""
Write-Host "Test keypair public key: $TEST_NPUB" -ForegroundColor Gray
Write-Host "You can verify the post on any Nostr client by searching for this npub" -ForegroundColor Gray
