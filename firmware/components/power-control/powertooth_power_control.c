#include "powertooth_power_control.h"

#include <string.h>
#include "driver/gpio.h"
#include "esp_check.h"
#include "esp_log.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#define POWER_SWITCH_GPIO ((gpio_num_t)CONFIG_POWERTOOTH_POWER_SWITCH_GPIO)
#define POWER_LED_SENSE_GPIO ((gpio_num_t)CONFIG_POWERTOOTH_POWER_LED_SENSE_GPIO)
#define CASE_BUTTON_GPIO ((gpio_num_t)CONFIG_POWERTOOTH_CASE_BUTTON_GPIO)
#define CASE_LED_GPIO ((gpio_num_t)CONFIG_POWERTOOTH_CASE_LED_GPIO)
#define ACTIVE_LEVEL(value) ((value) ? 1 : 0)

_Static_assert(CONFIG_POWERTOOTH_LED_ON_MIN_SAMPLES <= CONFIG_POWERTOOTH_LED_SAMPLE_COUNT,
               "Power LED on threshold cannot exceed sample count");
_Static_assert(CONFIG_POWERTOOTH_POWER_SWITCH_GPIO != CONFIG_POWERTOOTH_POWER_LED_SENSE_GPIO,
               "Power switch and LED sense GPIOs must be different");
_Static_assert(CONFIG_POWERTOOTH_CASE_BUTTON_GPIO != CONFIG_POWERTOOTH_CASE_LED_GPIO,
               "Case button and case LED GPIOs must be different");

static const char *TAG = "power";
#ifdef CONFIG_POWERTOOTH_DEBUG_LOGS
#define POWER_LOGI(format, ...) ESP_LOGI(TAG, format, ##__VA_ARGS__)
#else
#define POWER_LOGI(format, ...) do { } while (0)
#endif
static volatile bool pairing;
static volatile bool wake_pending;
static char pending_address[18];
static int64_t last_wake_us;
static powertooth_pair_callback_t pair_callback;

bool powertooth_power_pc_is_on(void) {
    int high_samples = 0;
    for (int i = 0; i < CONFIG_POWERTOOTH_LED_SAMPLE_COUNT; ++i) {
        high_samples += gpio_get_level(POWER_LED_SENSE_GPIO);
        vTaskDelay(pdMS_TO_TICKS(CONFIG_POWERTOOTH_LED_SAMPLE_INTERVAL_MS));
    }
    bool high_means_on = CONFIG_POWERTOOTH_POWER_LED_ACTIVE_HIGH;
    bool sampled_high = high_samples >= CONFIG_POWERTOOTH_LED_ON_MIN_SAMPLES;
    return high_means_on ? sampled_high : !sampled_high;
}

static void pulse(const char *reason) {
    POWER_LOGI("Power pulse: %s", reason);
    gpio_set_level(POWER_SWITCH_GPIO, ACTIVE_LEVEL(CONFIG_POWERTOOTH_POWER_SWITCH_ACTIVE_HIGH));
    vTaskDelay(pdMS_TO_TICKS(CONFIG_POWERTOOTH_POWER_PULSE_MS));
    gpio_set_level(POWER_SWITCH_GPIO, !ACTIVE_LEVEL(CONFIG_POWERTOOTH_POWER_SWITCH_ACTIVE_HIGH));
}

void powertooth_power_set_pair_callback(powertooth_pair_callback_t callback) {
    pair_callback = callback;
}

void powertooth_power_set_pairing(bool enabled) {
    pairing = enabled;
    if (enabled) POWER_LOGI("PAIRING");
}

void powertooth_power_request_wake(const char *address) {
    strlcpy(pending_address, address, sizeof(pending_address));
    wake_pending = true;
}

static void indication_task(void *unused) {
    (void)unused;
    int previous_output_level = -1;
    while (true) {
        if (pairing) {
            POWER_LOGI("Pairing: flashing %d times", CONFIG_POWERTOOTH_PAIR_FLASH_COUNT);
            for (int flash = 0;
                 flash < CONFIG_POWERTOOTH_PAIR_FLASH_COUNT && pairing;
                 ++flash) {
                gpio_set_level(CASE_LED_GPIO,
                               ACTIVE_LEVEL(CONFIG_POWERTOOTH_CASE_LED_ACTIVE_HIGH));
                vTaskDelay(pdMS_TO_TICKS(CONFIG_POWERTOOTH_PAIR_FLASH_MS));
                gpio_set_level(CASE_LED_GPIO,
                               !ACTIVE_LEVEL(CONFIG_POWERTOOTH_CASE_LED_ACTIVE_HIGH));
                vTaskDelay(pdMS_TO_TICKS(CONFIG_POWERTOOTH_PAIR_FLASH_MS));
            }
            if (pairing) {
                POWER_LOGI("Pairing: waiting %d ms", CONFIG_POWERTOOTH_PAIR_FLASH_PAUSE_MS);
                vTaskDelay(pdMS_TO_TICKS(CONFIG_POWERTOOTH_PAIR_FLASH_PAUSE_MS));
            }
            // Force a fresh steady-state log line once pairing mode ends.
            previous_output_level = -1;
            continue;
        }

        bool led_on = powertooth_power_pc_is_on();
        int output_level = led_on == CONFIG_POWERTOOTH_CASE_LED_ACTIVE_HIGH ? 1 : 0;
        gpio_set_level(CASE_LED_GPIO, output_level);
        if (output_level != previous_output_level) {
            POWER_LOGI("Case LED %s (GPIO %d level=%d)",
                       led_on ? "on" : "off", CASE_LED_GPIO, output_level);
            previous_output_level = output_level;
        }
        vTaskDelay(pdMS_TO_TICKS(CONFIG_POWERTOOTH_PAIR_FLASH_MS));
    }
}

static void control_task(void *unused) {
    (void)unused;
    bool previous_pressed = false;
    int64_t button_down_us = 0;
    while (true) {
        bool button_level = gpio_get_level(CASE_BUTTON_GPIO);
        bool pressed = CONFIG_POWERTOOTH_CASE_BUTTON_ACTIVE_LOW ? !button_level : button_level;
        if (!previous_pressed && pressed) {
            button_down_us = esp_timer_get_time();
            POWER_LOGI("Case button pressed (GPIO %d level=%d)",
                       CASE_BUTTON_GPIO, button_level);
        } else if (previous_pressed && !pressed) {
            int64_t held_ms = (esp_timer_get_time() - button_down_us) / 1000;
            POWER_LOGI("Case button released after %lld ms (GPIO %d level=%d)",
                       held_ms, CASE_BUTTON_GPIO, button_level);
            if (held_ms >= CONFIG_POWERTOOTH_PAIR_HOLD_MS) {
                POWER_LOGI("Case button long press; requesting pairing");
                powertooth_power_set_pairing(true);
                if (pair_callback) pair_callback();
            } else if (held_ms >= CONFIG_POWERTOOTH_BUTTON_DEBOUNCE_MS) {
                pulse("case button");
            }
        }
        previous_pressed = pressed;

        if (wake_pending) {
            wake_pending = false;
            int64_t now = esp_timer_get_time();
            if (last_wake_us == 0 ||
                now - last_wake_us >= CONFIG_POWERTOOTH_WAKE_COOLDOWN_MS * 1000LL) {
                if (powertooth_power_pc_is_on()) {
                    POWER_LOGI("Detected POWER ON; wake suppressed");
                } else {
                    POWER_LOGI("Controller %s requested wake", pending_address);
                    pulse("stored controller detected");
                    last_wake_us = esp_timer_get_time();
                }
            }
        }
        vTaskDelay(pdMS_TO_TICKS(20));
    }
}

esp_err_t powertooth_power_init(void) {
    gpio_config_t outputs = {
        .pin_bit_mask = (1ULL << POWER_SWITCH_GPIO) | (1ULL << CASE_LED_GPIO),
        .mode = GPIO_MODE_OUTPUT,
    };
    ESP_RETURN_ON_ERROR(gpio_config(&outputs), TAG, "configure outputs");
    gpio_set_level(POWER_SWITCH_GPIO, !ACTIVE_LEVEL(CONFIG_POWERTOOTH_POWER_SWITCH_ACTIVE_HIGH));
    gpio_set_level(CASE_LED_GPIO, !ACTIVE_LEVEL(CONFIG_POWERTOOTH_CASE_LED_ACTIVE_HIGH));

    gpio_config_t led_sense = {
        .pin_bit_mask = 1ULL << POWER_LED_SENSE_GPIO,
        .mode = GPIO_MODE_INPUT,
        .pull_down_en = GPIO_PULLDOWN_ENABLE,
    };
    ESP_RETURN_ON_ERROR(gpio_config(&led_sense), TAG, "configure LED sense");
    gpio_config_t button = {
        .pin_bit_mask = 1ULL << CASE_BUTTON_GPIO,
        .mode = GPIO_MODE_INPUT,
        .pull_up_en = GPIO_PULLUP_ENABLE,
    };
    ESP_RETURN_ON_ERROR(gpio_config(&button), TAG, "configure case button");
    POWER_LOGI("Case button input: GPIO %d, active-%s, initial level=%d",
               CASE_BUTTON_GPIO,
               CONFIG_POWERTOOTH_CASE_BUTTON_ACTIVE_LOW ? "low" : "high",
               gpio_get_level(CASE_BUTTON_GPIO));
    xTaskCreate(indication_task, "indication", 2048, NULL, 3, NULL);
    xTaskCreate(control_task, "power_control", 3072, NULL, 4, NULL);
    return ESP_OK;
}
