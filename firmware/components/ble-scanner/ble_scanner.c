#include "powertooth_ble.h"

#include "esp_bt.h"
#include "esp_bt_main.h"
#include "esp_gap_ble_api.h"
#include "esp_log.h"
#include "powertooth_power.h"
#include "powertooth_registry.h"

_Static_assert(CONFIG_POWERTOOTH_BLE_SCAN_WINDOW <= CONFIG_POWERTOOTH_BLE_SCAN_INTERVAL,
               "BLE scan window cannot exceed scan interval");

#ifdef CONFIG_POWERTOOTH_DEBUG_LOGS
static const char *TAG = "ble_scanner";
#define BLE_LOGI(format, ...) ESP_LOGI(TAG, format, ##__VA_ARGS__)
#else
#define BLE_LOGI(format, ...) do { } while (0)
#endif

static void gap_callback(esp_gap_ble_cb_event_t event, esp_ble_gap_cb_param_t *param) {
    if (event == ESP_GAP_BLE_SCAN_PARAM_SET_COMPLETE_EVT) {
        esp_ble_gap_start_scanning(0);
        return;
    }
    if (event != ESP_GAP_BLE_SCAN_RESULT_EVT ||
        param->scan_rst.search_evt != ESP_GAP_SEARCH_INQ_RES_EVT) return;

    char address[POWERTOOTH_ADDRESS_LENGTH];
    if (powertooth_address_format(param->scan_rst.bda, address) &&
        powertooth_registry_contains(address)) {
        BLE_LOGI("Known device recognized: %s RSSI=%d", address, param->scan_rst.rssi);
        powertooth_power_request_wake(address);
    }
}

esp_err_t powertooth_ble_init(void) {
    ESP_ERROR_CHECK(esp_bt_controller_mem_release(ESP_BT_MODE_CLASSIC_BT));
    esp_bt_controller_config_t config = BT_CONTROLLER_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_bt_controller_init(&config));
    ESP_ERROR_CHECK(esp_bt_controller_enable(ESP_BT_MODE_BLE));
    ESP_ERROR_CHECK(esp_bluedroid_init());
    ESP_ERROR_CHECK(esp_bluedroid_enable());
    ESP_ERROR_CHECK(esp_ble_gap_register_callback(gap_callback));
    static esp_ble_scan_params_t scan = {
        .scan_type = BLE_SCAN_TYPE_ACTIVE,
        .own_addr_type = BLE_ADDR_TYPE_PUBLIC,
        .scan_filter_policy = BLE_SCAN_FILTER_ALLOW_ALL,
        .scan_interval = CONFIG_POWERTOOTH_BLE_SCAN_INTERVAL,
        .scan_window = CONFIG_POWERTOOTH_BLE_SCAN_WINDOW,
        .scan_duplicate = BLE_SCAN_DUPLICATE_ENABLE,
    };
    return esp_ble_gap_set_scan_params(&scan);
}
