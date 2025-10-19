## TFRAlert
Alerts for new TFRs. Is now a good time to fly that drone? Let's check first.

### Installation
Prebuilt binaries for windows, mac, linux

Build from source: Clone repo. Install rust toolchain. `cargo build`. 

### Notes
- **Privacy note**: Makes periodic web requests to `tfr.faa.gov`. Use from public wifi, with a VPN, or via another proxy if that's a concern. 
- **The vibes the vibes the vibes**: Heavily AI-assisted code generation.
- **Best effort**: Data as fresh as `tfr.faa.gov` provides it. For a complete list of all TFRs by category, see that original JSON feed.

### Development
Built with [Dioxus](https://dioxuslabs.com/). Development builds: `dx serve`.


### Signing and notarizing macOS builds (WIP)

notarytool profile:
```
xcrun notarytool store-credentials "TFRAlertProfile" \
  --apple-id "signer-apple-id@example.com" \
  --team-id "TEAMID" \
  --password "app-specific-password"
```

export it:
```
export NOTARY_KEYCHAIN_PROFILE=TFRAlertProfile
```

sign and notarize:

```
export MACOS_IDENTITY="Developer ID Application: Signer Name (TEAMID)"
make notarize-all
```