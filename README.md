# Wave Jumper
Wild wavetables with your breadboard.

Weave demultiplexors and multiplexors together with jumper wires creating analogous path through the samples of a wav file.
Audio will be split into sub samples and mapped to each channel of the multimplexors.
A sample can be 'jumped' to by connecting the channels together.

This is part of my series in 'weird interfaces' where I try to regain that unexpected sound by forcing my brain through unfamiliar UI.

-------------

![20251022_113514](https://github.com/user-attachments/assets/79433203-55d7-4495-aedd-a1379072356b)

### In depth Write-Up
[Link to be added upon publish]()

### Demos
- https://youtu.be/X0Sj2huCUgs
- https://youtu.be/EcjNQMBe9rQ

### Dependencies
- [Rpi-Pal](https://github.com/rpi-pal/rpi-pal): Peripheral Access 🔌
- [Rodio](https://github.com/RustAudio/rodio): Audio 📢

### Install
```
sudo apt update && sudo apt upgrade
sudo apt install libasound2-dev pulseaudio pulseaudio-utils alsa-utils
git clone https://github.com/pixmusix/wave_jumper.git
cd wave_jumper
cargo build -- jobs 2 #we are on an SBC so max 2 tasks in parallel
```

## Hardware

### Board BOM

| SKU       | #    | DESC                                                 | LINK                                                                                  | Required |
|-----------|------|------------------------------------------------------|---------------------------------------------------------------------------------------|----------|
| CE06974   | 1    | Any Raspberry Pi Board (Tested on Pi 400 and Zero 2) | https://core-electronics.com.au/raspberry-pi.html#category_272                        | True     |
| CE04441   | 1    | RPi GPIO Breakout for breadboard.                    | https://core-electronics.com.au/40-pin-raspberry-pi-gpio-breakout.html                | False    |
| BOB-13906 | 4(+) | Sparkfun Multiplexor Breakout (or sub for 74HC4051)  | https://core-electronics.com.au/sparkfun-multiplexer-breakout-8-channel-74hc4051.html | True     |
| CEO9437   | 1    | Any SSD1306 oled display                             | https://core-electronics.com.au/ssd1306-oled-white-pre-soldered.html                  | False    |
| PRT-11367 | ~    | Solid Core Wire                                      | https://core-electronics.com.au/hook-up-wire-assortment-solid-core-22-awg.html        | True     |
| CE00304   | 2(+) | Large breadboard                                     | https://core-electronics.com.au/solderless-breadboard-830-tie-point-zy-102.html       | True     |
| CE09607   | ~    | Jumper wire M/M (any length)                         | https://core-electronics.com.au/male-to-male-dupont-line-40-pin-10cm-24awg.html       | True     |
| PRT-14460 | 1    | Button                                               | https://core-electronics.com.au/multicolor-buttons-4-pack.html                        | True     |
| CE07199   | ~    | Resistors                                            | https://core-electronics.com.au/components/resistors/through-hole.html                | True     |
| COM-08375 | 2    | Capacitors                                           | https://core-electronics.com.au/capacitor-ceramic-0-1uf.html                          | True     |
| CE08576   | ~    | Leds                                                 | https://core-electronics.com.au/makerverse-led-assortment-5pcs-1.html                 | False    |
| 74HC73    | 1    | JK Flipflop IC                                       | https://www.jaycar.com.au/74hc73-dual-jk-flip-flop-ic/p/ZC4829                        | False    |
| 2N2222    | 1(+) | PNP transistor (or sub for P100)                     | https://core-electronics.com.au/pn100-npn-multi-replacement-transistor.html           | False    |

### Schmatic
<img width="800" height="600" alt="unnamed" src="https://github.com/user-attachments/assets/a5217a36-98e8-4aec-b0fd-07ac5afb9567" />

### Music Demos
This project downloads with 5 tracks to get you started. You’ll find them all in the assets directory of the repository.
-  Arp - Creative Commons, LevelClearer
- Synth - Attribution 4.0, ImaTaco
- Home - Attribution 3.0, GrowingUp
- Jungle - Creative Commons (can’t find source 🙁)
- Bass - GPL 3.0, Pixmusix

