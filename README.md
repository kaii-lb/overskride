# Overskride
A Bluetooth and (soon to be) Obex client that is straight to the point, DE/WM agnostic, and beautiful (also soon to be) :D

![Screenshot](/assets/images/overskride.png)

# Prerequisites
- gtk4 and libadwaita (development packages)
- rust
- cargo
- bluez (should be installed by default on all distros)

# Compiling
- `git clone https://github.com/kaii-lb/overskride && cd overskride`
- `meson setup build && cd build`
- `meson compile && meson devenv`
- `mkdir -p ~/.local/share/glib-2.0/schemas`
- `cp ../data/com.kaii.Overskride.gschema.xml ~/.local/share/glib-2.0/schemas`
- `glib-compile-schemas ~/.local/share/glib-2.0/schemas`
- `./src/overskride`

###### this should be automated later on but oh well
###### press the refresh button to start discovering devices (will be automated)

# Features
- Dynamically enumerate and list all devices known/in range 
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

# What doesn't work
- **Authenticating with devices (aka passkey confirmation)**
- Sending/receiving files
- Audio profiles
- Auto accept files
- Changing files storage location

###### obviously the "what doesn't work" is a lot right now, but its in the works :D
###### oh and also the screenshot is themed, this isn't what normal adwaita looks like, if you want it to look like this go [here](https://github.com/kaii-lb/dotfiles) and then copy the thingies in `presets` to somewhere on your disk, and use gradience to apply the theme :D

