#!/bin/sh

cd /home/$USER/Projects/overskride/build/package

cp ../src/overskride overskride/usr/bin/overskride
cp ../src/overskride.gresource overskride/usr/share/overskride/
cp ../data/io.github.kaii_lb.Overskride.desktop overskride/usr/share/applications/
cp ../data/io.github.kaii_lb.Overskride.appdata.xml overskride/usr/share/appdata/
cp /home/kaii/Projects/overskride/data/io.github.kaii_lb.Overskride.gschema.xml overskride/usr/share/glib-2.0/schemas/
cp /home/kaii/Projects/overskride/data/icons/hicolor/scalable/apps/io.github.kaii_lb.Overskride.svg overskride/usr/share/icons/hicolor/scalable/apps/
cp /home/kaii/Projects/overskride/data/icons/hicolor/symbolic/apps/io.github.kaii_lb.Overskride-symbolic.svg overskride/usr/share/icons/hicolor/symbolic/apps/

7z a overskride.7z overskride/
cp overskride.7z ../../
