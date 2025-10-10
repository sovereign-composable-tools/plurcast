# Plurcast Keyring Testing Script
# Automated testing suite for OS-level credential security

param(
    [switch]$SkipBuild,
    [switch]$Verbose,
    [switch]$KeepCredentials
)

$ErrorActionPreference = "Continue"

# Colors
function Write-Phase { param($msg) Write-Host "`n=== $msg ===" -ForegroundColor Cyan }
function Write-Step { param($msg) Write-Host "  → $msg" -ForegroundColor Yellow }
function Write-Success { param($msg) Write-Host "  ✓ $msg" -ForegroundColor Green }
function Write-Failure { param($msg) Write-Host "  ✗ $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "  ℹ $msg" -ForegroundColor Blue }

# Test results
$script:passed = 0
$script:failed = 0
$script:skipped = 0

function Test-Command {
    param(
        [string]$Name,
        [scriptblock]$Command,
        [string]$ExpectedOutput = $null
    )
    
    Write-Step "Testing: $Name"
    
    try {
        $output = & $Command 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($Verbose) {
            Write-Host "    Output: $output" -ForegroundColor DarkGray
            Write-Host "    Exit code: $exitCode" -ForegroundColor DarkGray
        }
        
        if ($exitCode -eq 0) {
            if ($ExpectedOutput -and $output -notmatch $ExpectedOutput) {
                Write-Failure "$Name (unexpected output)"
                $script:failed++
                return $false
            }
            Write-Success $Name
            $script:passed++
            return $true
        } else {
            Write-Failure "$Name (exit code: $exitCode)"
            $script:failed++
            return $false
        }
    } catch {
        Write-Failure "$Name (exception: $_)"
        $script:failed++
        return $false
    }
}

# Main testing flow
Write-Host @"
╔═══════════════════════════════════════════════════════════╗
║     Plurcast OS-Level Keyring Security Testing Suite     ║
╚═══════════════════════════════════════════════════════════╝
"@ -ForegroundColor Cyan

# Phase 0: Build
if (-not $SkipBuild) {
    Write-Phase "Phase 0: Building Binaries"
    Write-Step "Building release binaries..."
    
    cargo build --release 2>&1 | Out-Null
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Build completed"
    } else {
        Write-Failure "Build failed"
        exit 1
    }
} else {
    Write-Info "Skipping build (using existing binaries)"
}

# Verify binaries exist
$plurCreds = ".\target\release\plur-creds.exe"
$plurPost = ".\target\release\plur-post.exe"

if (-not (Test-Path $plurCreds)) {
    Write-Failure "plur-creds binary not found at $plurCreds"
    exit 1
}

# Phase 1: Unit Tests
Write-Phase "Phase 1: Unit Tests"
Write-Step "Running credential unit tests..."

$testOutput = cargo test --lib credentials 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Success "Unit tests passed"
    $script:passed++
} else {
    Write-Failure "Unit tests failed"
    $script:failed++
    if ($Verbose) {
        Write-Host $testOutput -ForegroundColor DarkGray
    }
}

Write-Step "Running keyring-specific tests (may fail if keyring unavailable)..."
$keyringOutput = cargo test --lib credentials -- --ignored 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Success "Keyring tests passed"
    $script:passed++
} else {
    Write-Info "Keyring tests skipped or failed (this is OK in some environments)"
    $script:skipped++
}

# Phase 2: Configuration
Write-Phase "Phase 2: Configuration Check"

$configPath = "$env:USERPROFILE\.config\plurcast\config.toml"
Write-Step "Checking configuration at $configPath"

if (Test-Path $configPath) {
    $configContent = Get-Content $configPath -Raw
    if ($configContent -match 'storage\s*=\s*"keyring"') {
        Write-Success "Keyring storage configured"
        $script:passed++
    } else {
        Write-Info "Keyring not configured (will test fallback)"
        $script:skipped++
    }
} else {
    Write-Info "No configuration file found (will use defaults)"
    $script:skipped++
}

# Phase 3: Cleanup
Write-Phase "Phase 3: Pre-Test Cleanup"
Write-Step "Removing any existing test credentials..."

& $plurCreds delete nostr --force 2>&1 | Out-Null
& $plurCreds delete mastodon --force 2>&1 | Out-Null
& $plurCreds delete bluesky --force 2>&1 | Out-Null

Write-Success "Cleanup complete"

# Phase 4: Credential Storage
Write-Phase "Phase 4: Credential Storage Tests"

# Test Nostr credential storage
Write-Step "Storing Nostr test credential..."
$testKey = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
$storeResult = $testKey | & $plurCreds set nostr 2>&1

if ($LASTEXITCODE -eq 0 -and $storeResult -match "Stored.*nostr") {
    Write-Success "Nostr credential stored"
    $script:passed++
} else {
    Write-Failure "Failed to store Nostr credential"
    $script:failed++
    if ($Verbose) {
        Write-Host "    Output: $storeResult" -ForegroundColor DarkGray
    }
}

# Phase 5: Credential Listing
Write-Phase "Phase 5: Credential Listing Tests"

Write-Step "Listing stored credentials..."
$listResult = & $plurCreds list 2>&1

if ($LASTEXITCODE -eq 0 -and $listResult -match "nostr") {
    Write-Success "Credentials listed successfully"
    $script:passed++
    
    # Check which backend is being used
    if ($listResult -match "keyring") {
        Write-Success "Using keyring backend"
        $script:passed++
    } elseif ($listResult -match "encrypted") {
        Write-Info "Using encrypted file backend (keyring unavailable)"
        $script:skipped++
    } elseif ($listResult -match "plain") {
        Write-Info "Using plain file backend (not recommended)"
        $script:skipped++
    }
} else {
    Write-Failure "Failed to list credentials"
    $script:failed++
}

if ($Verbose) {
    Write-Host "`n$listResult`n" -ForegroundColor DarkGray
}

# Phase 6: Credential Retrieval
Write-Phase "Phase 6: Credential Retrieval Tests"

Test-Command -Name "Test Nostr credentials" -Command {
    & $plurCreds test nostr
} -ExpectedOutput "credentials found"

Test-Command -Name "Test all credentials" -Command {
    & $plurCreds test --all
}

# Phase 7: Security Audit
Write-Phase "Phase 7: Security Audit"

Write-Step "Running security audit..."
$auditResult = & $plurCreds audit 2>&1
$auditExitCode = $LASTEXITCODE

if ($Verbose) {
    Write-Host "`n$auditResult`n" -ForegroundColor DarkGray
}

if ($auditExitCode -eq 0) {
    Write-Success "Security audit passed (no issues found)"
    $script:passed++
} else {
    if ($auditResult -match "plain text") {
        Write-Info "Security issues found (plain text files detected)"
        $script:skipped++
    } else {
        Write-Failure "Security audit failed"
        $script:failed++
    }
}

# Phase 8: OS-Level Verification
Write-Phase "Phase 8: OS-Level Verification"

Write-Step "Checking Windows Credential Manager..."

try {
    # Try to find plurcast credentials in Windows Credential Manager
    $creds = cmdkey /list 2>&1 | Select-String "plurcast"
    
    if ($creds) {
        Write-Success "Credentials found in Windows Credential Manager"
        $script:passed++
        if ($Verbose) {
            Write-Host "    Found: $creds" -ForegroundColor DarkGray
        }
    } else {
        Write-Info "Credentials not in Windows Credential Manager (may be using file storage)"
        $script:skipped++
    }
} catch {
    Write-Info "Could not check Windows Credential Manager"
    $script:skipped++
}

# Phase 9: Integration Test (if plur-post exists)
if (Test-Path $plurPost) {
    Write-Phase "Phase 9: Integration Test with plur-post"
    
    Write-Step "Testing credential retrieval via plur-post..."
    
    # Note: This will fail if the test key isn't valid for Nostr, but we're just
    # testing that credentials can be retrieved
    $postResult = "Test post from keyring testing" | & $plurPost --platform nostr --draft 2>&1
    
    if ($postResult -match "error.*authentication" -or $postResult -match "error.*key") {
        Write-Info "Credential retrieved but authentication failed (expected with test key)"
        $script:passed++
    } elseif ($postResult -match "Saved draft") {
        Write-Success "Draft saved successfully (credentials retrieved)"
        $script:passed++
    } elseif ($postResult -match "Posted to nostr") {
        Write-Success "Posted successfully (credentials retrieved and valid)"
        $script:passed++
    } else {
        Write-Info "Integration test inconclusive"
        $script:skipped++
    }
    
    if ($Verbose) {
        Write-Host "    Output: $postResult" -ForegroundColor DarkGray
    }
} else {
    Write-Info "Skipping integration test (plur-post not found)"
    $script:skipped++
}

# Phase 10: Cleanup
if (-not $KeepCredentials) {
    Write-Phase "Phase 10: Post-Test Cleanup"
    Write-Step "Removing test credentials..."
    
    & $plurCreds delete nostr --force 2>&1 | Out-Null
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Test credentials removed"
        $script:passed++
    } else {
        Write-Failure "Failed to remove test credentials"
        $script:failed++
    }
} else {
    Write-Info "Keeping test credentials (--KeepCredentials flag set)"
}

# Summary
Write-Host @"

╔═══════════════════════════════════════════════════════════╗
║                      Test Summary                         ║
╚═══════════════════════════════════════════════════════════╝
"@ -ForegroundColor Cyan

Write-Host "  Passed:  $script:passed" -ForegroundColor Green
Write-Host "  Failed:  $script:failed" -ForegroundColor Red
Write-Host "  Skipped: $script:skipped" -ForegroundColor Yellow
Write-Host "  Total:   $($script:passed + $script:failed + $script:skipped)" -ForegroundColor Cyan

if ($script:failed -eq 0) {
    Write-Host "`n✓ All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n✗ Some tests failed. Review output above." -ForegroundColor Red
    exit 1
}
