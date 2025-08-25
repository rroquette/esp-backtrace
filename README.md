# esp-backtrace

A tool to parse and symbolize ESP32 backtraces from panic logs.
It extracts backtraces from logs, and uses `addr2line` to convert addresses to function names and source code locations.

## Usage

```sh
esp-backtrace --elf <path_to_elf> [--verbose] --file <log_file>
```

- `--elf <path_to_elf>`: Path to the ELF file containing debug symbols.
- `--file <log_file>`: Path to the log file containing backtraces.
- `--verbose`: Optional flag to print parsed backtraces before symbolization.

## Example output

```
Stacktrace #0 @ offset 192637 (2025-08-23 02:50:27 UTC)
SP- 0: 0x3fcb7c90 in panic_abort (/opt/esp/idf/components/esp_system/panic.c:454)
SP- 1: 0x3fcb7cb0 in esp_system_abort (/opt/esp/idf/components/esp_system/port/esp_system_chip.c:92)
SP- 2: 0x3fcb7cd0 in __assert_func (/opt/esp/idf/components/newlib/assert.c:80)
SP- 3: 0x3fcb7df0 in tlsf_free (/opt/esp/idf/components/heap/tlsf/tlsf.c:629)
SP- 4: 0x3fcb7e10 in multi_heap_free_impl (/opt/esp/idf/components/heap/multi_heap.c:233)
SP- 5: 0x3fcb7e30 in heap_caps_free (/opt/esp/idf/components/heap/heap_caps_base.c:70)
SP- 6: 0x3fcb7e50 in free (/opt/esp/idf/components/newlib/heap.c:39)
SP- 7: 0x3fcb7e70 in esp_tls_internal_event_tracker_destroy (/opt/esp/idf/components/esp-tls/esp_tls_error_capture.c:50)
SP- 8: 0x3fcb7e90 in esp_tls_conn_destroy (/opt/esp/idf/components/esp-tls/esp_tls.c:160)
SP- 9: 0x3fcb7eb0 in base_close (/opt/esp/idf/components/tcp_transport/transport_ssl.c:332)
SP-10: 0x3fcb7ed0 in esp_transport_close (/opt/esp/idf/components/tcp_transport/transport.c:172)
SP-11: 0x3fcb7ef0 in esp_mqtt_task (/opt/esp/idf/components/mqtt/esp-mqtt/mqtt_client.c:1743)
SP-12: 0x3fcb7f30 in vPortTaskWrapper (/opt/esp/idf/components/freertos/FreeRTOS-Kernel/portable/xtensa/port.c:139)

```
