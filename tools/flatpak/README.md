# Flatpak packages

Shanti Keyboard is distributed as an IBus Flatpak for GNOME. The direct
release bundles are named `shanti-ibus-x86_64.flatpak` for Intel/AMD and
`shanti-ibus-aarch64.flatpak` for ARM64. Download them from the
[GitHub Releases](https://github.com/hasanmuammar/Shanti-Keyboard/releases)
page.

## GTK rewrite development environment

The GTK rewrite has a separate development manifest at
`tools/flatpak/shanti-gtk-dev.yml`. It uses the GNOME 51 Platform and SDK,
includes the Rust SDK extension, and checks that GTK4 and libadwaita are
available inside the build SDK. It does not change the production IBus
manifest or release runtime.

Build the development app and run its CLI in the sandbox:

```sh
tools/flatpak/gtk-dev.sh build
tools/flatpak/gtk-dev.sh version
tools/flatpak/gtk-dev.sh check-data
tools/flatpak/gtk-dev.sh shell
```

The Flatpak build state and build directory live together under
`${XDG_CACHE_HOME:-$HOME/.cache}/shanti-keyboard/gtk-dev`, keeping them on the
same filesystem as required by Flatpak Builder. The first build needs Flathub
and its GNOME 51 runtime/SDK installed, plus network access to fetch any
missing Flatpak sources.

---

## ShantiIbus — the IBus engine (for GNOME)

Packages the Shanti settings GUI and the IBus engine as
`org.shanti.ShantiIbus`.

### Install from the hosted repository

After the first tagged release has published the repository, install Shanti
from its `.flatpakref` so Flatpak can receive future app updates from the
Shanti repository:

```sh
flatpak install --user \
  https://hasanmuammar.github.io/Shanti-Keyboard/shanti-ibus.flatpakref
```

The repository serves signed app refs and points to Flathub for the KDE
runtime. The direct bundle installation below remains available for offline
transfer or testing a specific release artifact.

After installing from either source, follow the setup steps below. Skip step 2
if you installed from the hosted `.flatpakref`.

### Install and add the GNOME input source

1. Make sure Flathub is configured for the same scope you will use for the app.
   For a per-user install:

   ```sh
   flatpak remote-add --user --if-not-exists flathub \
     https://flathub.org/repo/flathub.flatpakrepo
   ```

   The app needs the separate KDE runtime `org.kde.Platform//5.15-25.08`.
   Flatpak normally downloads it from Flathub during app installation. The
   bundle does not contain the runtime.

2. For a direct bundle installation, install the downloaded file. Replace the
   filename below with your downloaded bundle:

   ```sh
   flatpak install --user ./shanti-ibus-x86_64.flatpak
   ```

   If Flatpak says the runtime was not found, the remote and app are probably
   in different scopes. Either install the app with `--user` as above, or add
   Flathub and install the app system-wide. You can also install the runtime
   explicitly in the matching scope:

   ```sh
   flatpak install --user flathub org.kde.Platform//5.15-25.08
   ```

3. Launch **Shanti Keyboard (IBus)** from the app grid.
4. On first launch, accept **Set up integration**. Or open the app's
   **Settings** → **GNOME IBus Integration** and select **Set up / repair**.
   Confirm the action.
5. Open **GNOME Settings → Keyboard → Input Sources → Add Input Source**, then
   select **Shanti**.
6. Select Shanti from the input menu and try typing Bangla in another app.

The app's Settings panel reports whether its registration is present and
GNOME's IBus service is active. If an unmanaged IBus override blocks setup, the
app shows its path and offers **Back up and continue**. Choose this only if you
recognize the file as an old Shanti setup you want to replace; the app
preserves it as a backup before installing the current integration. Choose
**Cancel** if you do not recognize it. The integration manages user-level
files; it does not modify system files.

### Remove

1. In the app, open **Settings → GNOME IBus Integration → Remove integration**
   and confirm. This removes Shanti's GNOME registration. If the app is
   unavailable, see [uninstallation](../../docs/wiki/Uninstallation.md) for the
   managed file paths and manual cleanup guidance.
2. If **Shanti** remains in GNOME's input-source list, remove it in
   **GNOME Settings → Keyboard → Input Sources**.
3. Uninstall the app:

   ```sh
   flatpak uninstall --user org.shanti.ShantiIbus
   ```

Upgrading from a pre-rebrand release (`org.openbangla.Shanti.IBus`)? Install
the new bundle normally; on first run it copies your settings and user data
from the old `openbangla-keyboard` locations to the new `shanti-keyboard`
locations. After the upgrade you can remove the old app:
`flatpak uninstall --user org.openbangla.Shanti.IBus`.

## Verify a download (optional)

The release includes a `.sha256` file alongside each bundle. From the directory
containing both files, run:

```sh
sha256sum --check shanti-ibus-x86_64.flatpak.sha256
```

Use the aarch64 checksum filename for an ARM64 download. Releases also include
GitHub build-provenance attestations; see GitHub's
[artifact attestation verification](https://cli.github.com/manual/gh_attestation_verify)
documentation if you want to verify one with `gh`.

# Build locally

### Test a CI build from a branch or PR

Every push (including PR branches) builds the IBus Flatpak for x86_64 and
aarch64 and uploads downloadable artifacts named
`shanti-ibus-flatpak-x86_64` and `shanti-ibus-flatpak-aarch64`. Grab them with
`gh` using the run ID:

```sh
gh -R hasanmuammar/OpenBangla-Keyboard-Shanti run list --branch rust-rewrite
gh -R hasanmuammar/OpenBangla-Keyboard-Shanti run download <run-id> \
  -n shanti-ibus-flatpak-x86_64 -D /tmp/shanti-flatpak-test
flatpak install --user --reinstall /tmp/shanti-flatpak-test/shanti-ibus-x86_64.flatpak
```

Then follow [Install and add the GNOME input source](#install-and-add-the-gnome-input-source).
Log out and back in so IBus picks up the updated engine.

### Build from a checkout

To build and install the IBus bundle from a checkout, use Flatpak Builder on a
Linux host with Flatpak and Flatpak Builder installed:

```sh
flatpak-builder --user --install --force-clean \
  --default-branch=stable \
  /tmp/shanti-ibus-build tools/flatpak/shanti-ibus.yml
```

This downloads the build dependencies and can take several minutes. Then launch
it with:

```sh
flatpak run --user org.shanti.ShantiIbus
```

> Tip: `flatpak-builder` requires its state directory (default
> `.flatpak-builder` inside the checkout) to be on the **same filesystem** as
> the target directory. If your clone lives on a different filesystem than
> `/tmp`, pass a matching `--state-dir`:
>
> ```sh
> mkdir -p /tmp/opencode
> flatpak-builder --user --install --force-clean \
>   --state-dir=/tmp/opencode/flatpak-build-state \
>   --default-branch=stable \
>   /tmp/shanti-ibus-build tools/flatpak/shanti-ibus.yml
> ```

### Troubleshooting: build fails at `appstreamcli compose`

`flatpak-builder` finishes by running `appstreamcli compose`. If the
`appstreamcli` found on `PATH` was installed without its compose helper — for
example a Homebrew `appstream` on Bluefin, Bazzite and other atomic desktops —
the build fails near the end with:

```
AppStream Compose binary '…/libexec/appstreamcli-compose' was not found!
Error: appstreamcli compose failed
```

The helper path is compiled into `appstreamcli`. On Bluefin-style systems the
system helper already exists at `/usr/libexec/appstreamcli-compose`; expose it
where the Homebrew AppStream expects it and rebuild:

```sh
brew_cellar="$(brew --cellar appstream)/$(brew list --versions appstream | cut -d' ' -f2)"
mkdir -p "$brew_cellar/libexec"
ln -s /usr/libexec/appstreamcli-compose "$brew_cellar/libexec/appstreamcli-compose"
```

Alternatively, build with the sandboxed Flatpak Builder app, which does not
use the host's `appstreamcli`:

```sh
flatpak run org.flatpak.Builder --user --install --force-clean \
  --default-branch=stable \
  /tmp/shanti-ibus-build tools/flatpak/shanti-ibus.yml
```

The release bundles are direct downloads, not a Flatpak remote. If installation
says the required runtime was not found, check that Flathub is added to the
same installation scope. Then run the explicit runtime install above, omitting
`--user` for a system-wide install. The Flatpak and the portable installer
(from the source checkout's `tools/build.sh`) are separate installations.
