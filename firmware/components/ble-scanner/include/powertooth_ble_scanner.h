#pragma once
#include "esp_err.h"

// Continiuous BLE scanning. TODO: refactor such that this scanning is only active when the computer is shutdown.
esp_err_t powertooth_ble_init(void);