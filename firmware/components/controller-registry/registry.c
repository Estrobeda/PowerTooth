#include "powertooth_registry.h"

#include <ctype.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "nvs.h"

#define NVS_NAMESPACE "powertooth"

static char devices[CONFIG_POWERTOOTH_MAX_DEVICES][POWERTOOTH_ADDRESS_LENGTH];
static size_t device_count;
static SemaphoreHandle_t lock;

static bool normalize(const char *input, char output[POWERTOOTH_ADDRESS_LENGTH]) {
    if (strlen(input) != 17) return false;
    for (size_t i = 0; i < 17; ++i) {
        if ((i + 1) % 3 == 0) {
            if (input[i] != ':') return false;
            output[i] = ':';
        } else {
            if (!isxdigit((unsigned char)input[i])) return false;
            output[i] = (char)tolower((unsigned char)input[i]);
        }
    }
    output[17] = '\0';
    return true;
}

static int find_locked(const char *address) {
    for (size_t i = 0; i < device_count; ++i) {
        if (strcmp(devices[i], address) == 0) return (int)i;
    }
    return -1;
}

static esp_err_t save_locked(void) {
    nvs_handle_t handle;
    esp_err_t error = nvs_open(NVS_NAMESPACE, NVS_READWRITE, &handle);
    if (error != ESP_OK) return error;
    error = nvs_set_blob(handle, "devices", devices, sizeof(devices));
    if (error == ESP_OK) error = nvs_set_u8(handle, "count", (uint8_t)device_count);
    if (error == ESP_OK) error = nvs_commit(handle);
    nvs_close(handle);
    return error;
}

esp_err_t powertooth_registry_init(void) {
    lock = xSemaphoreCreateMutex();
    if (!lock) return ESP_ERR_NO_MEM;
    nvs_handle_t handle;
    if (nvs_open(NVS_NAMESPACE, NVS_READONLY, &handle) != ESP_OK) return ESP_OK;
    size_t length = sizeof(devices);
    uint8_t count = 0;
    if (nvs_get_blob(handle, "devices", devices, &length) == ESP_OK &&
        length == sizeof(devices) && nvs_get_u8(handle, "count", &count) == ESP_OK &&
        count <= CONFIG_POWERTOOTH_MAX_DEVICES) {
        device_count = count;
    }
    nvs_close(handle);
    return ESP_OK;
}

size_t powertooth_registry_count(void) {
    xSemaphoreTake(lock, portMAX_DELAY);
    size_t count = device_count;
    xSemaphoreGive(lock);
    return count;
}

esp_err_t powertooth_registry_get(size_t index, char address[POWERTOOTH_ADDRESS_LENGTH]) {
    xSemaphoreTake(lock, portMAX_DELAY);
    if (index >= device_count) {
        xSemaphoreGive(lock);
        return ESP_ERR_NOT_FOUND;
    }
    strlcpy(address, devices[index], POWERTOOTH_ADDRESS_LENGTH);
    xSemaphoreGive(lock);
    return ESP_OK;
}

bool powertooth_registry_contains(const char *input) {
    char address[POWERTOOTH_ADDRESS_LENGTH];
    if (!normalize(input, address)) return false;
    xSemaphoreTake(lock, portMAX_DELAY);
    bool found = find_locked(address) >= 0;
    xSemaphoreGive(lock);
    return found;
}

esp_err_t powertooth_registry_add(const char *input) {
    char address[POWERTOOTH_ADDRESS_LENGTH];
    if (!normalize(input, address)) return ESP_ERR_INVALID_ARG;
    xSemaphoreTake(lock, portMAX_DELAY);
    if (find_locked(address) >= 0) {
        xSemaphoreGive(lock);
        return ESP_OK;
    }
    if (device_count >= CONFIG_POWERTOOTH_MAX_DEVICES) {
        xSemaphoreGive(lock);
        return ESP_ERR_NO_MEM;
    }
    strlcpy(devices[device_count++], address, POWERTOOTH_ADDRESS_LENGTH);
    esp_err_t error = save_locked();
    xSemaphoreGive(lock);
    return error;
}

esp_err_t powertooth_registry_remove(const char *input) {
    char address[POWERTOOTH_ADDRESS_LENGTH];
    if (!normalize(input, address)) return ESP_ERR_INVALID_ARG;
    xSemaphoreTake(lock, portMAX_DELAY);
    int index = find_locked(address);
    if (index >= 0) {
        for (size_t i = (size_t)index; i + 1 < device_count; ++i) {
            memcpy(devices[i], devices[i + 1], POWERTOOTH_ADDRESS_LENGTH);
        }
        memset(devices[--device_count], 0, POWERTOOTH_ADDRESS_LENGTH);
    }
    esp_err_t error = save_locked();
    xSemaphoreGive(lock);
    return error;
}

esp_err_t powertooth_registry_reset(void) {
    xSemaphoreTake(lock, portMAX_DELAY);
    memset(devices, 0, sizeof(devices));
    device_count = 0;
    esp_err_t error = save_locked();
    xSemaphoreGive(lock);
    return error;
}

bool powertooth_address_format(const uint8_t bytes[6], char output[POWERTOOTH_ADDRESS_LENGTH]) {
    return snprintf(output, POWERTOOTH_ADDRESS_LENGTH, "%02x:%02x:%02x:%02x:%02x:%02x",
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]) == 17;
}
