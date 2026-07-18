#pragma once

#include <stdbool.h>
#include "esp_err.h"

typedef void (*powertooth_pair_callback_t)(void);

esp_err_t powertooth_power_init(void);
void powertooth_power_set_pair_callback(powertooth_pair_callback_t callback);
void powertooth_power_set_pairing(bool enabled);
bool powertooth_power_pc_is_on(void);
void powertooth_power_request_wake(const char *address);

