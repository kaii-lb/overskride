#!/bin/sh

rm -rf flatpak-build/
rm -rf overskride-flatpak
mkdir flatpak-build/
mkdir overskride-flatpak

flatpak-builder flatpak-build/ io.github.kaii_lb.Overskride.json --force-clean
flatpak build-export overskride-flatpak flatpak-build main
flatpak build-bundle overskride-flatpak/ overskride.flatpak io.github.kaii_lb.Overskride main
