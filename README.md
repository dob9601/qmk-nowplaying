[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](https://makeapullrequest.com)
# QMK - Now Playing 🎵

This project allows for information about the currently playing song to be shown on a QMK keyboard's OLED display. It has only been tested on Linux.

The code in this repo is currently in a pretty hacky state as it serves as a demonstration of the capabilities of the [qmk-oled-api repository](https://github.com/dob9601/qmk-oled-api).

## Installation

Currently no packaging has been setup. Run the project via `cargo run` after setting the path to your device (`/dev/hidrawX` where `X` is some number). You will need to configure your keyboard as a `qmk-oled-api` client. An example of how to do so can be found in the README of the [qmk-oled-api repository](https://github.com/dob9601/qmk-oled-api#client-snippet)
