#include "powertooth_ble_scanner.h"

#include <string.h>

#include "esp_log.h"
#include "host/ble_gap.h"
#include "host/ble_hs.h"
#include "host/util/util.h"
#include "nimble/nimble_port.h"
#include "nimble/nimble_port_freertos.h"
#include "powertooth_power_control.h"
#include "powertooth_registry.h"

_Static_assert(CONFIG_POWERTOOTH_BLE_SCAN_WINDOW <= CONFIG_POWERTOOTH_BLE_SCAN_INTERVAL,
               "BLE scan window cannot exceed scan interval");

#ifdef CONFIG_POWERTOOTH_DEBUG_LOGS
static const char *TAG = "ble_scanner";
#define BLE_LOGI(format, ...) ESP_LOGI(TAG, format, ##__VA_ARGS__)
#else
#define BLE_LOGI(format, ...) do { } while (0)
#endif

static int gap_event(struct ble_gap_event *event, void *argument) {
    (void)argument;
    if (event->type != BLE_GAP_EVENT_DISC) return 0;

    /* NimBLE stores Bluetooth addresses least-significant byte first. */
    uint8_t address_bytes[6];
    for (size_t index = 0; index < sizeof(address_bytes); ++index) {
        address_bytes[index] = event->disc.addr.val[sizeof(address_bytes) - 1 - index];
    }

    char address[POWERTOOTH_ADDRESS_LENGTH];
    if (powertooth_address_format(address_bytes, address) &&
        powertooth_registry_contains(address)) {
        BLE_LOGI("Known device recognized: %s RSSI=%d", address, event->disc.rssi);
        powertooth_power_request_wake(address);
    }
    return 0;
}

static void start_scan(void) {
    uint8_t own_address_type;
    if (ble_hs_util_ensure_addr(0) != 0 ||
        ble_hs_id_infer_auto(0, &own_address_type) != 0) {
        return;
    }

    const struct ble_gap_disc_params parameters = {
        .itvl = CONFIG_POWERTOOTH_BLE_SCAN_INTERVAL,
        .window = CONFIG_POWERTOOTH_BLE_SCAN_WINDOW,
        .filter_policy = BLE_HCI_SCAN_FILT_NO_WL,
        .limited = 0,
        .passive = 1,
        .filter_duplicates = 1,
    };
    ble_gap_disc(own_address_type, BLE_HS_FOREVER, &parameters, gap_event, NULL);
}

static void host_task(void *argument) {
    (void)argument;
    nimble_port_run();
    nimble_port_freertos_deinit();
}

esp_err_t powertooth_ble_init(void) {
    esp_err_t result = nimble_port_init();
    if (result != ESP_OK) return result;

    ble_hs_cfg.sync_cb = start_scan;
    nimble_port_freertos_init(host_task);
    return ESP_OK;
}
