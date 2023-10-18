# Overskride
A Bluetooth and Obex client that is straight to the point, DE/WM agnostic, and beautiful :D

![Screenshot](/assets/images/overskride.png)

# Prerequisites
- gtk4 and libadwaita (development packages)
- rust
- cargo
- bluez (should be installed by default on all distros)

# How to install
```bash
sudo systemctl enable --now bluetooth
curl -sSL https://github.com/kaii-lb/overskride/releases/latest/download/overskride.flatpak -o ~/Downloads/overskride.flatpak
sudo flatpak install -y ~/Downloads/overskride.flatpak
rm ~/Downloads/overskride.flatpak
```

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

# What doesn't work
- Audio profiles
- Auto accept files
- Changing files storage location
- Battery polling over bluetooth


