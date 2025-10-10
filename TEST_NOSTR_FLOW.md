# Nostr Test Flow - First Time Testing

This document walks through testing Plurcast with a fresh Nostr test account.

## Prerequisites

- Plurcast built successfully (`cargo build --release`)
- No existing configuration (fresh start)

## Test Flow

### Step 1: Generate Test Nostr Keys

We'll use a simple method to generate test keys. For production, you'd use a proper Nostr client or tool like `nak`.

**Option A: Use an online tool (for testing only)**
- Visit: https://nostr-keygen.com/ or similar
- Generate a new keypair
- Save the private key (nsec format or hex)

**Option B: Use nak (if installed)**
```bash
nak key generate
```

**Option C: Let Plurcast generate (if implemented)**
```bash
./target/release/plur-setup
```

### Step 2: Create Configuration Directory

```powershell
# Create config directory
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.config\plurcast"

# Create data directory
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.local\share\plurcast"
```

### Step 3: Create Minimal Config File

Create `~/.config/plurcast/config.toml`:

```toml
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
```

### Step 4: Create Test Keys File

Create `~/.config/plurcast/nostr.keys` with your test private key:

```powershell
# Create the file (replace with your actual test key)
"YOUR_TEST_PRIVATE_KEY_HERE" | Out-File -FilePath "$env:USERPROFILE\.config\plurcast\nostr.keys" -NoNewline -Encoding utf8
```

**Important**: Use a TEST key, not your real Nostr identity!

### Step 5: Test Basic Posting

```powershell
# Test 1: Simple post
.\target\release\plur-post.exe "Test post from Plurcast - please ignore"

# Expected output: nostr:note1abc123...
# Exit code: 0 (success)

# Test 2: Post from stdin
echo "Another test post" | .\target\release\plur-post.exe

# Test 3: Verbose mode to see what's happening
.\target\release\plur-post.exe "Verbose test" --verbose

# Test 4: Draft mode (save without posting)
.\target\release\plur-post.exe "Draft test" --draft

# Expected output: draft:uuid
```

### Step 6: Check History

```powershell
# View recent posts
.\target\release\plur-history.exe

# JSON format
.\target\release\plur-history.exe --format json

# Filter by platform
.\target\release\plur-history.exe --platform nostr
```

### Step 7: Verify on Nostr

To verify your test posts actually made it to Nostr:

1. Get your public key (npub):
   - If you have `nak`: `nak key public <your-private-key>`
   - Or use an online converter

2. Check your posts on a Nostr client:
   - https://snort.social
   - https://iris.to
   - https://nostrudel.ninja
   - Search for your npub

### Step 8: Test Error Handling

```powershell
# Test empty content
echo "" | .\target\release\plur-post.exe
# Expected: Error, exit code 3

# Test oversized content
python -c "print('x'*100001)" | .\target\release\plur-post.exe
# Expected: Content too large error, exit code 3

# Test with invalid keys (temporarily rename keys file)
Move-Item "$env:USERPROFILE\.config\plurcast\nostr.keys" "$env:USERPROFILE\.config\plurcast\nostr.keys.bak"
.\target\release\plur-post.exe "Test"
# Expected: Authentication error, exit code 2
Move-Item "$env:USERPROFILE\.config\plurcast\nostr.keys.bak" "$env:USERPROFILE\.config\plurcast\nostr.keys"
```

## Expected Results

### Success Indicators

- ✅ Post returns `nostr:note1...` format ID
- ✅ Exit code 0 on success
- ✅ Posts visible on Nostr clients within 1-2 minutes
- ✅ History shows posts with timestamps
- ✅ Database created at `~/.local/share/plurcast/posts.db`

### Common Issues

**"Authentication failed: Could not read Nostr keys file"**
- Check file path in config.toml
- Verify file exists: `Test-Path "$env:USERPROFILE\.config\plurcast\nostr.keys"`
- Check file isn't empty

**"Invalid private key format"**
- Ensure key is 64-char hex OR nsec format
- Remove any whitespace/newlines
- Try both formats if unsure

**"Failed to connect to relay"**
- Check internet connection
- Try different relays
- Use `--verbose` to see connection attempts
- Plurcast succeeds if ANY relay accepts the post

**Posts don't appear on Nostr clients**
- Wait 1-2 minutes for propagation
- Check you're searching for the correct npub
- Try different Nostr clients
- Verify relays are working

## Cleanup After Testing

```powershell
# Remove test configuration
Remove-Item -Recurse -Force "$env:USERPROFILE\.config\plurcast"

# Remove test database
Remove-Item -Recurse -Force "$env:USERPROFILE\.local\share\plurcast"

# Or keep for future testing
```

## Next Steps

Once Nostr testing is successful:

1. Test Mastodon integration (requires OAuth token)
2. Test Bluesky integration (requires app password)
3. Test multi-platform posting
4. Test scheduling features (when implemented)

## Security Reminder

🔒 **These are TEST credentials**
- Do NOT use your real Nostr identity for testing
- Test keys can be discarded after testing
- Real credentials should use secure storage (keyring/encrypted)
- See SECURITY.md for production credential management

---

**Test Date**: _____________
**Tester**: _____________
**Result**: ☐ Pass ☐ Fail ☐ Partial
**Notes**:
