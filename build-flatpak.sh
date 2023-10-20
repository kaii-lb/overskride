#!/bin/sh

flatpak-builder flatpak-build/ io.github.kaii_lb.Overskride.json --force-clean
flatpak build-export overskride-flatpak flatpak-build main
flatpak build-bundle overskride-flatpak/ overskride.flatpak io.github.kaii_lb.Overskride main
