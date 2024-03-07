# Rust ESP32-C3 BME280

Example of reading the HX711 24-bit analog-to-digital converter and a load cell sensor on an ESP32-C3 using Rust

## Hardware:

- [ESP32-C3](https://mm.digikey.com/Volume0/opasdata/d220001/medias/images/3824/ESP32-DEVKITM-1.jpg)
- [HX711](https://grobotronics.com/images/detailed/117/htb1fepyipxxxxx.xvxxq6xxfxxxe_grobo.jpg)
- [Load cell](https://cdn.sparkfun.com/assets/learn_tutorials/3/8/2/13329-01Crop.jpg)
- Breadboard
- Jump wires
- USB-A to Micro-B cable
- [Weighing acrylic](https://www.elecbee.com/image/catalog/Sensor-and-Detector-Module/ESP32-096-OLED-HX711-Digital-Load-Cell-1KG-Weight-Sensor-Board-Development-Tool-Kit-1410870-descriptionImage11.jpeg)

Step 1: solder the load cell wires to the HX711: 

- Red to E+
- Black to E-
- White to A-
- Green to A+

Step 2 (optional): screw the weighing acrylics to the load sensor. 

Step 3 solder headers to the HX711's GND, DT, SCK, and VCC. 

Step 4: attach the HX711 to the breadboard (with its headers).

Step 5: attach the ESP32-C3 to the breadboard (with its headers).

Step 6: using the jump wires, connect the ESP32-C3 to the HX711: 

- 3V3 to VCC
- GND to GND
- GPIO 2 to DT
- GPIO 3 to SCK

Note: you can use any available GPIO pins for DT and SCK, but remember to change the code accordingly.

Step 7: using the USB cable, connect the ESP32-C3 to your computer or laptop.

The final setup should look like [this](https://i0.wp.com/randomnerdtutorials.com/wp-content/uploads/2022/03/ESP32-load-cell-diagram_bb.png?resize=828%2C382&quality=100&strip=all&ssl=1).

## Software:

Step 1: follow these [instructions](https://github.com/esp-rs/esp-idf-template?tab=readme-ov-file#prerequisites) to setup the development environment.

Step 2: execute `cargo run` on the command line (to build, flash, and monitor). Note: on Linux, you may have to fix a permission [issue](https://github.com/esp-rs/espflash/blob/main/espflash/README.md#permissions-on-linux).

Step 3: calibrate the load cell following [these](https://github.com/DaneSlattery/hx711?tab=readme-ov-file#calibration) instructions (you can ask ChatGPT to do the math). 


If the universe conspires in your favor, you should see an output like this (after applying pressure on the load sensor):

```bash
I (78145) rust_esp32_hx711: Weight: 0 g
I (79145) rust_esp32_hx711: Weight: 45 g
```

Let me know if I skipped any step.

