#include "esp_err.h"
#include "esp_log.h"
#include "nvs_flash.h"
#include "powertooth_ble.h"
#include "powertooth_host_link.h"
#include "powertooth_power.h"
#include "powertooth_registry.h"

static void initialize_nvs(void) {
    esp_err_t error = nvs_flash_init();
    if (error == ESP_ERR_NVS_NO_FREE_PAGES || error == ESP_ERR_NVS_NEW_VERSION_FOUND) {
        ESP_ERROR_CHECK(nvs_flash_erase());
        error = nvs_flash_init();
    }
    ESP_ERROR_CHECK(error);
}

void app_main(void) {
    initialize_nvs();
    ESP_ERROR_CHECK(powertooth_registry_init());
    ESP_ERROR_CHECK(powertooth_power_init());
    ESP_ERROR_CHECK(powertooth_ble_init());
    ESP_ERROR_CHECK(powertooth_host_link_start());
}
