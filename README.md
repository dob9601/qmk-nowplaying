[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](https://makeapullrequest.com)
# QMK - Now Playing 🎵
Show information and album art for the currently playing song on the OLED screen of your QMK keyboard
![IMG_20220814_150825-01](https://user-images.githubusercontent.com/24723950/184541002-cbaf09e5-dd6f-4e44-b6d8-9febc8c89cbb.jpeg)


This project allows for information about the currently playing song to be shown on a QMK keyboard's OLED display.

It is currently untested on Windows but is unlikely to work due to dependence on MPRIS2-compatible media players. Pull Requests are welcome to extend support though

The code in this repo is currently in a pretty hacky state as it serves as a demonstration of the capabilities of the [qmk-oled-api repository](https://github.com/dob9601/qmk-oled-api).

## Installation

Currently no packaging has been setup. Run the project via `cargo run` after setting the path to your device (`/dev/hidrawX` where `X` is some number). You will need to configure your keyboard as a `qmk-oled-api` client. An example of how to do so can be found in the README of the [qmk-oled-api repository](https://github.com/dob9601/qmk-oled-api#client-snippet)

pre-built binaries are [available in releases](https://github.com/dob9601/qmk-nowplaying/releases/latest)
