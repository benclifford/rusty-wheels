Rusty Wheels
============

Persistence of Vision bike wheel LEDS, using a Raspberry Pi Zero, Rust and 46
RGB LEDs.

Software
--------

This repo, on top of Raspbian.

Download this font
gttps://gitlab.freedesktop.org/xorg/font/misc-misc/-/blob/master/5x7.bdf and
save it as font.bdf in the working directory where the code runs.

Hardware
--------

* Raspberry Pi Zero
* 46 DotStar RGB LEDs, cut into two strips, one for each side of the wheel
* Hall effect magnetic sensor: Hobbytronics A1120EUA-T Hall Effect Switch
* Magnet
* Level shifter: v1 = SparkFun BOB-12009 (v2, I'm hacking around with
  PCA9306 https://www.sparkfun.com/products/15439)
* Perma-proto board
* Screw terminals x 7
* Buttons x 3

GPIO connections:

* Hall effect sensor: GPIO27
* DotStar LEDs (SPI) via level shifter: MOSI and CLK
* Buttons: Smart power button: between SCL and 0v. Other two buttons to GPIO12,
  GPIO13

Screw terminals:

* 3 for hall effect sensor
* 4 for dotstar LEDs
