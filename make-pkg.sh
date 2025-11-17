#!/bin/sh

meson compile -C build

mkdir -p /home/$USER/Projects/overskride/build/src/release/package
cd /home/$USER/Projects/overskride/build/src/release/package

mkdir -p usr/bin
mkdir -p usr/share/overskride
mkdir -p usr/share/applications
mkdir -p usr/share/appdata
mkdir -p usr/share/glib-2.0/schemas
mkdir -p usr/share/icons/hicolor/scalable/apps
mkdir -p usr/share/icons/hicolor/symbolic/apps

cp ../../overskride usr/bin/overskride
cp ../../overskride.gresource usr/share/overskride/
cp ../../../data/io.github.kaii_lb.Overskride.desktop usr/share/applications/
cp ../../../data/io.github.kaii_lb.Overskride.appdata.xml usr/share/appdata/
cp /home/$USER/Projects/overskride/data/io.github.kaii_lb.Overskride.gschema.xml usr/share/glib-2.0/schemas/
cp /home/$USER/Projects/overskride/data/icons/hicolor/scalable/apps/io.github.kaii_lb.Overskride.svg usr/share/icons/hicolor/scalable/apps/
cp /home/$USER/Projects/overskride/data/icons/hicolor/symbolic/apps/io.github.kaii_lb.Overskride-symbolic.svg usr/share/icons/hicolor/symbolic/apps/

cd ..

tar -cf - package/ | xz -9 -T0 > overskride.tar.xz
cp overskride.tar.xz ../../../
