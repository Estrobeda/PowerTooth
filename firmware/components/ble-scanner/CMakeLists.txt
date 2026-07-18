if(CONFIG_BT_NIMBLE_ENABLED)
    set(POWERTOOTH_BLE_SCANNER_SOURCE "powertooth_ble_scanner_nimble.c")
else()
    set(POWERTOOTH_BLE_SCANNER_SOURCE "powertooth_ble_scanner_bluedroid.c")
endif()

idf_component_register(SRCS "${POWERTOOTH_BLE_SCANNER_SOURCE}" INCLUDE_DIRS "include"
    REQUIRES bt controller-registry power-control)
