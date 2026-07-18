# Hardware interface

Default ESP32-C3/C6 assignments (all are configurable under `idf.py menuconfig` → **PowerTooth**):

| GPIO | Function | Electrical interface |
|---|---|---|
| 3 | Case button input | Button to ground, internal pull-up |
| 4 | Motherboard power-LED sense | Optocoupler or protected 3.3 V logic input |
| 5 | Motherboard power-switch output | Active-high transistor/optocoupler driver |
| 6 | Case power-LED output | Current-limited LED driver or transistor stage |

Do not connect motherboard headers directly to an ESP32 GPIO. Motherboards vary in polarity, voltage, and whether header pins share ground. Use an optocoupler for `PWR_SW` when possible and verify every pin with a multimeter.

## Intended signal path

```text
Motherboard PLED header -> protected/isolated sense -> GPIO 4
GPIO 6 -> current-limited driver -> case power LED

Case power button -> GPIO 3 and ground
GPIO 5 -> transistor/optocoupler -> motherboard PWR_SW header
```

This routing lets firmware mirror the normal motherboard LED state, flash the case LED during pairing, and pass short case-button presses to the motherboard. A long press of at least two seconds enters pairing mode without sending a power pulse.

The ESP32 must be powered from a rail that remains available while the PC is off. Confirm this behavior before installation.

## Configuration

Run `idf.py menuconfig` and open **PowerTooth**. GPIO numbers, all four active polarities, power-pulse duration, cooldown, long-press time, debounce time, LED sampling, pairing flash rate, registry capacity, and BLE scan interval/window are build configuration—not source constants.

The host's serial path and baud rate are runtime options. Native USB Serial/JTAG does not use a physical baud rate, but UART-console board variants must also set ESP-IDF's **Console UART baud rate** to the same value passed through `powertooth --baud`.

## Power-state interpretation

By default, GPIO 4 is sampled twenty times over two seconds. Eight or more high samples mean `POWER ON`; this tolerates some motherboard LED blinking. These values are starting points and must be calibrated against the actual motherboard's sleep and shutdown patterns.
