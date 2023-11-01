# Overskride
A Bluetooth and Obex client that is straight to the point, DE/WM agnostic, and beautiful :D

![Screenshot](/assets/images/overskride.png)

# How to install (recommended)
- run `sudo systemctl enable --now bluetooth`
- download the `.flatpak` from the latest build in the [the github actions page](https://github.com/kaii-lb/overskride/actions/)
- save it to `~/Downloads/overskride-nightly.flatpak`
- if needed, run `sudo flatpak install org.gnome.Platform//45`
- double click the `.flatpak` or run `sudo flatpak install ~/Downloads/overskride-nightly.flatpak`
- profit

or you could: 

```bash
sudo systemctl enable --now bluetooth
curl -sSL https://nightly.link/kaii-lb/overskride/workflows/main/v0.5.3/overskride-nightly-x86_64.zip -o ~/Downloads/overskride-nightly.zip
unzip ~/Downloads/overskride-nightly.zip -d ~/Downloads/
sudo flatpak install org.gnome.Platform//45 # only if needed
sudo flatpak install -y ~/Downloads/overskride-nightly.flatpak
rm ~/Downloads/overskride-nightly.flatpak
```

# Major releases (old, not the newest features)
available at [releases page]( https://github.com/kaii-lb/overskride/releases/latest)

# Prerequisites for building
- gtk4 and libadwaita (development packages)
- rust
- cargo
- bluez (should be installed by default on all distros)

# Compiling
```bash
git clone https://github.com/kaii-lb/overskride && cd overskride
meson setup build && cd build
meson compile && meson devenv
mkdir -p ~/.local/share/glib-2.0/schemas
cp ../data/io.github.kaii_lb.Overskride.gschema.xml ~/.local/share/glib-2.0/schemas
glib-compile-schemas ~/.local/share/glib-2.0/schemas
./src/overskride
```

###### this should be automated later on but oh well

# Features
- Dynamically enumerate and list all devices known/in range 
- Authenticating with devices (aka passkey confirmation)
- Sending/receiving files
- Connect/disconnect from devices
- Rename device
- Trust or block a device
- Remove device
- Turn adapter on/off
- Set discoverable and its timeout
- Selecting between multiple adapters
- Rename adapter 
- Resizing support 
- Sorting devices by rssi (signal strength)
- Showing errors to user
- Changing files storage location
- Auto accept files
- Audio profiles
- Battery polling over bluetooth (enable experimental bluetooth options)
- Transfer rate for incoming/outgoing file transfers

# What doesn't work
- Applet support aka system tray
- More info about device
