#include "powertooth_host_link.h"

#include <stdio.h>
#include <string.h>
#include "esp_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "powertooth_power.h"
#include "powertooth_registry.h"

#ifdef CONFIG_POWERTOOTH_DEBUG_LOGS
static const char *TAG = "host_link";
#define HOST_LOGW(format, ...) ESP_LOGW(TAG, format, ##__VA_ARGS__)
#else
#define HOST_LOGW(format, ...) do { } while (0)
#endif

static void send_line(const char *body) {
    printf("PT/1 %s\n", body);
    fflush(stdout);
}

void powertooth_host_link_send_pair_request(void) {
    send_line("PAIR");
}

static void send_result(esp_err_t error) {
    if (error == ESP_OK) send_line("OK");
    else if (error == ESP_ERR_INVALID_ARG) send_line("ERR invalid-address");
    else if (error == ESP_ERR_NO_MEM) send_line("ERR registry-full");
    else send_line("ERR storage");
}

static void list_devices(void) {
    size_t count = powertooth_registry_count();
    for (size_t i = 0; i < count; ++i) {
        char address[POWERTOOTH_ADDRESS_LENGTH];
        if (powertooth_registry_get(i, address) == ESP_OK) {
            char response[48];
            snprintf(response, sizeof(response), "DEVICE %s", address);
            send_line(response);
        }
    }
    send_line("END");
}

static void handle(char *line) {
    if (strncmp(line, "PT/1 ", 5) != 0) return;
    char *command = line + 5;
    if (strcmp(command, "HELLO") == 0) send_line("OK");
    else if (strcmp(command, "LIST") == 0) list_devices();
    else if (strncmp(command, "ADD ", 4) == 0) send_result(powertooth_registry_add(command + 4));
    else if (strncmp(command, "REMOVE ", 7) == 0) send_result(powertooth_registry_remove(command + 7));
    else if (strcmp(command, "RESET") == 0) send_result(powertooth_registry_reset());
    else if (strcmp(command, "SYNC") == 0) {
        powertooth_power_set_pairing(false);
        send_line("OK");
    } else if (strcmp(command, "POWER?") == 0) {
        send_line(powertooth_power_pc_is_on() ? "POWER ON" : "POWER OFF");
    } else {
        HOST_LOGW("Unknown command length=%u value='%s'", (unsigned)strlen(command), command);
        send_line("ERR unknown-command");
    }
}

static void task(void *unused) {
    (void)unused;
    char line[96];
    size_t length = 0;
    while (true) {
        int byte = fgetc(stdin);
        if (byte == EOF) {
            clearerr(stdin);
            vTaskDelay(pdMS_TO_TICKS(10));
            continue;
        }
        if (byte == '\r' || byte == '\n') {
            if (length > 0) {
                line[length] = '\0';
                handle(line);
                length = 0;
            }
        } else if (length + 1 < sizeof(line)) {
            line[length++] = (char)byte;
        } else {
            length = 0;
            send_line("ERR line-too-long");
        }
    }
}

esp_err_t powertooth_host_link_start(void) {
    powertooth_power_set_pair_callback(powertooth_host_link_send_pair_request);
    return xTaskCreate(task, "host_link", 4096, NULL, 5, NULL) == pdPASS ? ESP_OK : ESP_ERR_NO_MEM;
}
