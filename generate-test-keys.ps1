# Generate Test Nostr Keys for Plurcast Testing
# This creates a random test keypair for testing purposes only

Write-Host "🔑 Generating Test Nostr Keys..." -ForegroundColor Cyan
Write-Host ""
Write-Host "⚠️  WARNING: These are TEST keys only!" -ForegroundColor Yellow
Write-Host "   Do NOT use for your real Nostr identity" -ForegroundColor Yellow
Write-Host ""

# Generate random 32 bytes (256 bits) for private key
$bytes = New-Object byte[] 32
$rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
$rng.GetBytes($bytes)

# Convert to hex string
$privateKeyHex = ($bytes | ForEach-Object { $_.ToString("x2") }) -join ''

Write-Host "Generated Test Private Key (hex format):" -ForegroundColor Green
Write-Host $privateKeyHex
Write-Host ""

# Create config directory if it doesn't exist
$configDir = "$env:USERPROFILE\.config\plurcast"
if (-not (Test-Path $configDir)) {
    New-Item -ItemType Directory -Force -Path $configDir | Out-Null
    Write-Host "✓ Created config directory: $configDir" -ForegroundColor Green
}

# Save to keys file
$keysFile = "$configDir\nostr.keys"
$privateKeyHex | Out-File -FilePath $keysFile -NoNewline -Encoding utf8

Write-Host "✓ Saved to: $keysFile" -ForegroundColor Green
Write-Host ""

# Create minimal config if it doesn't exist
$configFile = "$configDir\config.toml"
if (-not (Test-Path $configFile)) {
    $configContent = @"
[database]
path = "~/.local/share/plurcast/posts.db"

[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]

[defaults]
platforms = ["nostr"]
"@
    $configContent | Out-File -FilePath $configFile -Encoding utf8
    Write-Host "✓ Created config file: $configFile" -ForegroundColor Green
    Write-Host ""
}

# Create data directory
$dataDir = "$env:USERPROFILE\.local\share\plurcast"
if (-not (Test-Path $dataDir)) {
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
    Write-Host "✓ Created data directory: $dataDir" -ForegroundColor Green
    Write-Host ""
}

Write-Host "🎉 Setup Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Test posting: .\target\release\plur-post.exe 'Test post - please ignore'"
Write-Host "2. View history: .\target\release\plur-history.exe"
Write-Host "3. See TEST_NOSTR_FLOW.md for full testing guide"
Write-Host ""
Write-Host "To find your public key (npub), you'll need a Nostr tool like 'nak'" -ForegroundColor Yellow
Write-Host "Or use an online converter: https://nostr.band/tools.html" -ForegroundColor Yellow
