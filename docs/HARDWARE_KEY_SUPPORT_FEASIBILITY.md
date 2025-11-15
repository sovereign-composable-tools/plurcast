# Hardware Key Support Feasibility Study

**Date**: 2025-11-15
**Status**: Research
**Priority**: Low (Future Enhancement)

---

## Executive Summary

**Question**: Should Plurcast support hardware signing devices (YubiKey, Ledger) for Nostr private keys?

**Answer**: ⚠️ **Defer to Phase 4** (Long-term feature)

**Rationale**:
- ✅ Technically feasible with community tools
- ⚠️ Limited ecosystem support (early stage)
- ⚠️ High implementation complexity
- ⚠️ User setup burden
- ✅ Best-in-class security for high-value accounts

**Recommendation**:
1. Implement memory protection first (Priority 1)
2. Add NIP-46 remote signing (Priority 3)
3. Evaluate hardware key support after ecosystem matures (Priority 4)

---

## Hardware Key Options

### 1. YubiKey

**Manufacturer**: Yubico
**Models**: YubiKey 5 series (firmware 5.2.3+)
**Ed25519 Support**: ✅ Yes (via OpenPGP 3.4)

#### Current Nostr Support

**Community Tools**:
- NIP-07 browser extensions that integrate with YubiKey
- Uses OS keychain + YubiKey for key protection
- Primarily browser-focused (Chrome extensions)

**Evidence**:
> "A NIP-07 browser extension that uses the OS's keychain or YubiKey to protect your private keys for Nostr"
>
> Source: GitHub topics (YubiKey + Nostr + NIP-07)

#### Technical Capabilities

**What YubiKey Can Do**:
- Store ed25519 private keys on-chip
- Perform signing operations on-device
- Keys never leave hardware
- OpenPGP card functionality
- PIV smart card support

**Firmware Requirements**:
- YubiKey 5.2.3+: Ed25519 support via OpenPGP 3.4
- YubiKey 5 Nano: Same capabilities in nano form factor

#### Integration Approach for Plurcast

**Option A: PIV (Personal Identity Verification)**

```rust
// Pseudo-code using yubikey crate
use yubikey::{YubiKey, PivSlot, certificate};

pub struct YubiKeyNostrSigner {
    yubikey: YubiKey,
    slot: PivSlot,
}

impl YubiKeyNostrSigner {
    pub fn new() -> Result<Self> {
        // Connect to YubiKey
        let yubikey = YubiKey::open()?;

        Ok(Self {
            yubikey,
            slot: PivSlot::Signature, // Slot 9C for signing
        })
    }

    pub fn sign_event(&mut self, event_bytes: &[u8]) -> Result<Signature> {
        // Sign using YubiKey's on-chip private key
        let signature = self.yubikey.sign_data(
            self.slot,
            event_bytes,
            SignatureAlgorithm::ECDSA_SHA256_P256, // Note: Ed25519 via PIV is complex
        )?;

        Ok(signature)
    }

    pub fn get_public_key(&self) -> Result<PublicKey> {
        // Retrieve public key from certificate
        let cert = self.yubikey.fetch_certificate(self.slot)?;
        let public_key = cert.subject_pki();
        Ok(public_key)
    }
}
```

**Challenges**:
- ❌ PIV primarily supports NIST P-256, not Ed25519 natively
- ❌ Ed25519 via PIV requires custom implementation
- ❌ Complex certificate management
- ⚠️ May need to use OpenPGP card mode instead

**Option B: OpenPGP Card**

```rust
// Pseudo-code using gpgme or openpgp-card
use openpgp_card::{Card, Algorithm};

pub struct YubiKeyOpenPGPSigner {
    card: Card,
}

impl YubiKeyOpenPGPSigner {
    pub fn new() -> Result<Self> {
        let card = Card::open()?;

        // Verify Ed25519 is supported
        if !card.supports_algorithm(Algorithm::Ed25519)? {
            return Err("YubiKey firmware too old (need 5.2.3+)");
        }

        Ok(Self { card })
    }

    pub fn sign_nostr_event(&mut self, event_id: &[u8; 32]) -> Result<[u8; 64]> {
        // Sign using EdDSA (Ed25519)
        let signature = self.card.sign(event_id, Algorithm::Ed25519)?;
        Ok(signature.try_into()?)
    }

    pub fn get_nostr_pubkey(&self) -> Result<[u8; 32]> {
        // Get public key from signing key slot
        let pubkey = self.card.public_key(KeySlot::Signing)?;
        Ok(pubkey.as_bytes().try_into()?)
    }
}
```

**Challenges**:
- ⚠️ Requires GPG toolchain (gpg-agent)
- ⚠️ User must initialize YubiKey in OpenPGP mode
- ⚠️ PIN entry for each signing operation (UX friction)
- ✅ Better Ed25519 support than PIV

**Verdict**: ⚠️ **Feasible but Complex**

---

### 2. Ledger

**Manufacturer**: Ledger SAS
**Models**: Ledger Nano S, Nano S Plus, Nano X
**Ed25519 Support**: ✅ Yes (native Bitcoin/crypto support)

#### Current Nostr Support

**Community Tools**:
- **Ledgstr**: Third-party Ledger app for Nostr key management
  - GitHub: `b0l0k/ledgstr-app`
  - Manages Nostr keys on Ledger device
  - Chrome extension with NIP-07 support
- Not official Ledger support (community project)

**Evidence**:
> "Ledgstr - Ledger application to manage your Nostr key in a secure way. A basic Chrome extension compatible with NIP-07 and communicating with the Ledgstr app"
>
> Source: GitHub (b0l0k/ledgstr-app, b0l0k/ledgstr-extension-chrome)

#### Technical Capabilities

**What Ledger Can Do**:
- Native Ed25519 signing (for Bitcoin, Solana, etc.)
- Secure element storage
- Screen for transaction verification
- BIP32/BIP39 key derivation

#### Integration Approach for Plurcast

**Option A: Ledgstr Integration**

```rust
// Pseudo-code using Ledgstr via USB HID
use ledger_transport_hid::{TransportNativeHID, LedgerHIDError};

pub struct LedgstrNostrSigner {
    transport: TransportNativeHID,
}

impl LedgstrNostrSigner {
    pub fn new() -> Result<Self> {
        // Connect to Ledger device
        let transport = TransportNativeHID::new()?;

        Ok(Self { transport })
    }

    pub fn sign_event(&mut self, event_hash: &[u8; 32]) -> Result<[u8; 64]> {
        // Send APDU command to Ledgstr app
        // Format: CLA | INS | P1 | P2 | Lc | Data
        let apdu = build_sign_apdu(event_hash);

        let response = self.transport.exchange(&apdu)?;

        // Parse signature from response
        parse_signature(response)
    }

    pub fn get_public_key(&mut self, derivation_path: &str) -> Result<[u8; 32]> {
        let apdu = build_get_pubkey_apdu(derivation_path);
        let response = self.transport.exchange(&apdu)?;
        parse_pubkey(response)
    }
}

fn build_sign_apdu(event_hash: &[u8; 32]) -> Vec<u8> {
    // Ledgstr APDU format (hypothetical, need actual spec)
    let mut apdu = vec![
        0xE0, // CLA (Ledgstr app class)
        0x02, // INS (sign instruction)
        0x00, // P1
        0x00, // P2
        32,   // Lc (data length)
    ];
    apdu.extend_from_slice(event_hash);
    apdu
}
```

**Challenges**:
- ❌ Ledgstr is third-party, not official Ledger app
- ❌ Not in Ledger Live app store (needs sideloading)
- ❌ APDU command format not well-documented
- ⚠️ User must manually install Ledgstr app on device
- ⚠️ Requires USB access permissions

**Option B: Custom Ledger App**

Could develop an official Plurcast Ledger app, but:
- ❌ High development effort (C code, Ledger SDK)
- ❌ Ledger app review process (months)
- ❌ Maintenance burden (firmware updates)
- ❌ Small user base for Nostr + CLI + Ledger

**Verdict**: ⚠️ **Feasible but High Barrier**

---

### 3. OneKey (Alternative)

**Manufacturer**: OneKey
**Nostr Support**: ✅ Official support announced

**Evidence**:
> "OneKey Hardware Wallet Announces Support for Lightning Network and Nostr... becoming the first encryption hardware wallet to support Lightning Network and Nostr. OneKey will support the 'nostr.signSchnorr' function"
>
> Source: CoinLive, 2025

#### Why OneKey is Promising

**Advantages**:
- ✅ Official Nostr support (not third-party)
- ✅ Native `nostr.signSchnorr` function
- ✅ Designed for Nostr + Lightning use case
- ✅ Open-source firmware

**Challenges**:
- ⚠️ Less established than YubiKey/Ledger
- ⚠️ Smaller user base
- ⚠️ SDK maturity unknown

**Verdict**: ⏰ **Watch and Evaluate**

---

## Implementation Complexity

### Required Dependencies

**For YubiKey**:
```toml
yubikey = "0.7"  # YubiKey PIV/OpenPGP support
pcsc = "2.8"     # Smart card interface
openpgp-card = "0.4"  # OpenPGP card protocol
```

**For Ledger**:
```toml
ledger-transport-hid = "0.10"  # USB HID transport
ledger-apdu = "0.10"           # APDU command building
```

**For OneKey**:
```toml
# Hypothetical, would need to research actual SDK
onekey-sdk = "?"
```

### Platform Support

| Feature | YubiKey | Ledger | OneKey |
|---------|---------|--------|--------|
| **Linux** | ✅ (via pcsc) | ✅ (via libusb) | ⚠️ Unknown |
| **macOS** | ✅ (native) | ✅ (native) | ⚠️ Unknown |
| **Windows** | ✅ (native) | ✅ (native) | ⚠️ Unknown |
| **Docker/Headless** | ❌ (needs USB) | ❌ (needs USB) | ❌ (needs USB) |

### User Setup Complexity

**YubiKey Setup**:
1. Purchase YubiKey 5 (firmware 5.2.3+)
2. Install YubiKey Manager
3. Set YubiKey to OpenPGP mode
4. Generate or import Ed25519 key
5. Set PIN codes
6. Export public key for Nostr

**Ledger Setup**:
1. Purchase Ledger Nano S/X
2. Install Ledgstr app (sideload, not in Ledger Live)
3. Initialize Nostr key on device
4. Install Ledgstr Chrome extension (for NIP-07)
5. Configure derivation path
6. Export public key

**OneKey Setup**:
1. Purchase OneKey device
2. Follow official Nostr setup guide
3. Use built-in Nostr signing function
4. Export public key

**Complexity Ranking**:
1. 🟢 OneKey: Simplest (official support)
2. 🟡 YubiKey: Medium (mature tooling)
3. 🔴 Ledger: Hardest (third-party app)

---

## Security Comparison

### Security Model

| Device | Key Storage | Signing | Screen | Audit |
|--------|-------------|---------|--------|-------|
| **YubiKey** | Secure element | On-chip | ❌ No | ✅ Open (PIV), ⚠️ Closed (secure element) |
| **Ledger** | Secure element | On-chip | ✅ Yes | ⚠️ Closed source |
| **OneKey** | Secure element | On-chip | ✅ Yes | ✅ Open source firmware |

### Attack Resistance

| Attack Vector | YubiKey | Ledger | OneKey | Software (Current) |
|--------------|---------|--------|--------|-------------------|
| **Memory dump** | ✅ N/A (no software key) | ✅ N/A | ✅ N/A | ❌ Vulnerable |
| **Malware** | ✅ Requires physical access | ✅ Requires physical access | ✅ Requires physical access | ⚠️ Can steal key |
| **Supply chain** | ⚠️ Medium (Yubico trusted) | ⚠️ Medium (Ledger breaches) | ⚠️ Unknown | ✅ N/A |
| **Physical theft** | ⚠️ PIN protects (3-8 retries) | ⚠️ PIN protects (3 retries) | ⚠️ PIN protects | ✅ N/A (no device) |
| **Phishing** | ✅ Can't extract key | ✅ Can't extract key + screen | ✅ Can't extract key + screen | ⚠️ Can trick user |

**Best for Security**: Ledger/OneKey (screen for transaction verification)
**Best for Usability**: Software keys (current approach)
**Best Balance**: YubiKey (good security, better UX than Ledger)

---

## Cost-Benefit Analysis

### Development Cost

**Effort Estimate**:
- YubiKey support: 3-4 weeks (full-time)
- Ledger support: 4-6 weeks (full-time)
- OneKey support: 2-3 weeks (if SDK available)
- Testing across platforms: 2 weeks
- Documentation and UX: 1 week

**Total**: 8-13 weeks (2-3 months full-time)

### User Cost

**Hardware Purchase**:
- YubiKey 5 NFC: $55-65 USD
- Ledger Nano S Plus: $79 USD
- Ledger Nano X: $149 USD
- OneKey: $50-100 USD (estimated)

**Setup Time**:
- YubiKey: 30-60 minutes
- Ledger: 60-90 minutes
- OneKey: 20-40 minutes (estimated)

### Benefit

**Who Needs Hardware Keys?**

**High-Value Users** (would benefit):
- Journalists using Nostr for censorship-resistant publishing
- Bitcoin influencers with large followings
- Corporate accounts (brands, companies)
- High-net-worth individuals concerned about targeted attacks

**Typical Users** (adequate with software):
- Casual Nostr users
- Developers testing Plurcast
- Automated bots/agents
- CI/CD systems

**Market Size**: Estimated <5% of Plurcast users would use hardware keys

---

## Alternatives to Hardware Keys

### 1. Memory Protection (Priority 1)

**Security**: ⭐⭐⭐⭐ (4/5)
**Effort**: 2-3 weeks
**Protects Against**: Memory dumps, swap files, crash dumps
**Verdict**: ✅ **Implement First**

### 2. NIP-46 Remote Signing (Priority 3)

**Security**: ⭐⭐⭐⭐⭐ (5/5)
**Effort**: 4-6 weeks
**Protects Against**: All software attacks (key on different device)
**Verdict**: ✅ **Better ROI than Hardware Keys**

**Why NIP-46 > Hardware Keys for Plurcast**:
- Works with any device (not just $50-150 hardware)
- Can use user's phone as signing device
- Native Nostr protocol (NIP-46)
- No USB requirements (works over Nostr relays)
- Better for CLI context (no screen on YubiKey)

### 3. Encrypted Key Files + OS Keyring (Current)

**Security**: ⭐⭐⭐⭐ (4/5 with memory protection)
**Effort**: Already implemented
**Protects Against**: File theft, unauthorized access
**Verdict**: ✅ **Sufficient for Most Users**

---

## Ecosystem Maturity

### Nostr Hardware Signing Ecosystem

**Current State** (as of 2025-11):
- ⚠️ **Early Stage**: Limited adoption
- ⚠️ **Fragmented**: Multiple competing approaches
- ⚠️ **Browser-Focused**: Most tools are NIP-07 extensions
- ⚠️ **Undocumented**: Sparse documentation for CLI integration

**What Exists**:
- YubiKey + NIP-07 browser extensions (experimental)
- Ledgstr (third-party Ledger app, not widely adopted)
- OneKey (announced, maturity unknown)

**What's Missing**:
- ❌ Standard protocol for hardware signing (no NIP for it)
- ❌ CLI-focused hardware integration
- ❌ Mature Rust libraries for Nostr hardware signing
- ❌ Testing infrastructure for hardware devices

**Recommendation**: ⏰ **Wait for Ecosystem to Mature**

---

## Recommended Timeline

### Phase 1: Memory Protection (Now)
**Timeframe**: Weeks 1-3
- Implement `secrecy` crate for key zeroing
- Add Drop handlers for platforms
- Security testing

**Security Improvement**: ⭐⭐⭐⭐ (4/5) → ⭐⭐⭐⭐⭐ (4.5/5)

### Phase 2: Documentation (Week 4)
- Security best practices guide
- Threat model documentation
- User education on key protection

### Phase 3: NIP-46 Remote Signing (Months 2-3)
**Timeframe**: 6-8 weeks
- Implement NIP-46 bunker support
- CLI can connect to remote signing service
- Better than hardware keys for most users

**Security Improvement**: ⭐⭐⭐⭐⭐ (5/5) for remote-key users

### Phase 4: Hardware Key Support (Months 6-12)
**Timeframe**: 8-13 weeks
**Conditions**:
- ✅ Ecosystem has matured (standard protocols)
- ✅ User demand validated (>50 requests)
- ✅ Rust libraries available
- ✅ OneKey or official Ledger support exists

**Approach** (when ready):
1. Start with OneKey (if SDK is good)
2. Add YubiKey via OpenPGP
3. Add Ledger if official app available

---

## Conclusion

### Should Plurcast Support Hardware Keys?

**Answer**: ⏰ **Not Yet, But Eventually**

### Recommended Priority

1. **Priority 1** (Now): Memory protection with `secrecy` crate
2. **Priority 2** (Month 1): Security documentation
3. **Priority 3** (Months 2-3): NIP-46 remote signing
4. **Priority 4** (Months 6-12): Hardware key support (if ecosystem matures)

### Why Defer Hardware Keys?

**Technical Reasons**:
- ⚠️ Ecosystem immature (no standard CLI protocols)
- ⚠️ Limited library support in Rust
- ⚠️ High implementation complexity
- ⚠️ Platform compatibility challenges (USB, permissions)

**Business Reasons**:
- ⏸️ Small market (<5% of users)
- 💰 High development cost (2-3 months)
- ⚠️ User setup friction ($50-150 + 1 hour setup)
- ✅ Better alternatives exist (NIP-46, memory protection)

**Security Reasons**:
- ✅ Memory protection + OS keyring is 90% as good
- ✅ NIP-46 achieves same security goal with less friction
- ✅ Diminishing returns vs effort invested

### When to Revisit

**Triggers to Re-evaluate**:
1. ✅ OneKey releases stable Rust SDK with good docs
2. ✅ Community develops standard NIP for hardware signing
3. ✅ User demand increases (>50 requests for hardware keys)
4. ✅ Mature Rust library emerges for Nostr hardware signing
5. ✅ Ledger adds official Nostr app to Ledger Live

**Monitor**:
- OneKey firmware releases and Nostr integration
- Ledgstr adoption and stability
- Nostr NIPs related to hardware signing
- Rust crate ecosystem (yubikey, ledger-transport, etc.)

---

## Action Items

### Immediate
- [x] Document hardware key research findings
- [ ] Add "Hardware Key Support" to roadmap (Priority 4)
- [ ] Monitor OneKey and Ledgstr development

### Short-term (Months 1-3)
- [ ] Implement memory protection (Priority 1)
- [ ] Implement NIP-46 support (Priority 3)
- [ ] Create security comparison docs for users

### Long-term (Months 6-12)
- [ ] Re-evaluate hardware key ecosystem maturity
- [ ] If ecosystem mature, prototype with OneKey
- [ ] User survey: hardware key demand validation

---

## References

### Yubico Documentation
- [YubiKey 5.2.3 OpenPGP Enhancements](https://developers.yubico.com/PGP/YubiKey_5.2.3_Enhancements_to_OpenPGP_3.4.html)
- [YubiKey OpenPGP support](https://www.yubico.com/blog/whats-new-in-yubikey-firmware-5-2-3/)

### Ledger Resources
- [Ledgstr App (GitHub)](https://github.com/b0l0k/ledgstr-app)
- [Ledgstr Chrome Extension](https://github.com/b0l0k/ledgstr-extension-chrome)

### OneKey
- [OneKey Nostr Announcement](https://www.coinlive.com/news-flash/410634)

### Nostr NIPs
- [NIP-07: Browser Extension Signing](https://github.com/nostr-protocol/nips/blob/master/07.md)
- [NIP-46: Nostr Connect (Remote Signing)](https://github.com/nostr-protocol/nips/blob/master/46.md)

### Rust Crates
- [`yubikey` crate](https://docs.rs/yubikey/)
- [`ledger-transport-hid` crate](https://docs.rs/ledger-transport-hid/)
- [`pcsc` crate](https://docs.rs/pcsc/)

---

**Document Status**: ✅ Research Complete
**Decision**: Defer to Phase 4 (Long-term)
**Next Review**: Q2 2026 or when ecosystem matures
