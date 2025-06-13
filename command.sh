# Source the ESP environment (note the correct path)
source /home/jaykchen/export-esp.sh

# Verify it worked
echo $LIBCLANG_PATH

# Unset IDF_PATH to let esp-idf-sys handle ESP-IDF
unset IDF_PATH

# Clean and build
cargo clean
cargo +esp build --target xtensa-esp32s3-espidf --release


unset IDF_PATH
unset ESP_IDF_PATH
unset ESP_IDF_VERSION
unset ESP_IDF_TOOLS_INSTALL_DIR


echo $IDF_PATH
echo $ESP_IDF_PATH
echo $ESP_IDF_VERSION
echo $ESP_IDF_TOOLS_INSTALL_DIR