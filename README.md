# Rust ESP32-C3 HX711 SUPABASE

Example of using Rust to read the HX711 24-bit analog-to-digital converter and a load cell sensor on an ESP32-C3, then send the readings to [Supabase](https://supabase.com/).

Step 1: Set up the HX711 and load cell following [these](https://github.com/weiying-chen/rust-esp32c3-hx711) instructions. 

Step 2: Add your Wi-Fi SSID (network name), Wi-Fi password, Supabase project URL, and Supabase API key. 

Step 3: Turn on your Wi-Fi hotspot.

Step 4: Create a table in Supabase with the `reading` (text) column.

Step 5: Execute `cargo run` on the command line (to build, flash, and monitor). Note: on Linux, you may have to fix a permission [issue](https://github.com/esp-rs/espflash/blob/main/espflash/README.md#permissions-on-linux).

Step 6: Check your table. You should see the readings there.

Note: make sure your ESP32-C3 is properly powered as Wi-Fi consumes a lot of power.

