#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "esp_err.h"
#include "sdkconfig.h"

#define POWERTOOTH_ADDRESS_LENGTH 18

esp_err_t powertooth_registry_init(void);
size_t powertooth_registry_count(void);
esp_err_t powertooth_registry_get(size_t index, char address[POWERTOOTH_ADDRESS_LENGTH]);
bool powertooth_registry_contains(const char *address);
esp_err_t powertooth_registry_add(const char *address);
esp_err_t powertooth_registry_remove(const char *address);
esp_err_t powertooth_registry_reset(void);
bool powertooth_address_format(const uint8_t bytes[6], char output[POWERTOOTH_ADDRESS_LENGTH]);
