## TFRAlert
Collect and filter TFRs/NOTAMs. Is it a good time to be out flying drones? Let's find out.

- TFRs/NOTAMs that restrict flights under 400ft AGL with specific keyword filtering
- Details of matching results are saved to a json file
- (Soon) periodic list refresh

- **Toy project**: This is an educational project and should not replace the due diligence practices of anyone actually operating any aircraft! See LICENSE.md.
- **Privacy note**: Makes periodic web requests to `tfr.faa.gov`. Use from behind a VPN or proxy or from a public WIFI network if that's a concern.
- **The vibes the vibes the vibes**: Heavily AI-assisted code generation.
- **Best effort**: Data as fresh as `tfr.faa.gov` provides it. For a complete list of all TFRs by category, see that original JSON feed.

![Screenshot of app UI](screenshots/tfr_ui.png)
![Screenshot of system notificatoin](screenshots/notif.png)

### Installation
**Under Construction**: Native linux support; Not yet ready for cross-platform installation. These instructions will be updated.

Installation via prebuilt desktop binaries will be supported for windows, mac(silicon and intel), and linux.

Build from source: See Development below for setup instructions. Install desired toolchain target, per Makefile, then run `make build-$TARGET`.

### Development
Clone repo. Install Rust toolchain and [Dioxus](https://dioxuslabs.com/). Note that this project currently uses a pre-release version of Dioxus, 0.7.0-rc2, which must be installed with `cargo binstall dioxus-cli@0.7.0-rc.2 --force` (this documentation will be updated).

Use `cargo run` to test local changes. `dx serve` (the dioxus-cli) is not currently supported but is planned.

### Signing and notarizing macOS builds: TODO
