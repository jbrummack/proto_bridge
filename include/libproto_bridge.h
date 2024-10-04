#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

typedef void (*callback_fn)(const uint8_t* data, size_t len, void* user_data);

void process_proto(const uint8_t* data, size_t len, callback_fn callback, void* user_data);
