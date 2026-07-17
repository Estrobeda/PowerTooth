idf_component_register(SRCS "main.c" INCLUDE_DIRS "."
    REQUIRES controller-registry host-link ble-scanner power-control nvs_flash)
