// Wrapper to generate Rust bindings via bindgen.

// FreeRTOS kernel includes:
#include <FreeRTOS.h>
#include <atomic.h>
#include <croutine.h>

#include <list.h>
#include <message_buffer.h>
#include <mpu_wrappers.h>
#include <event_groups.h>
#include <projdefs.h>
#include <queue.h>
#include <semphr.h>
#include <stack_macros.h>
#include <stream_buffer.h>
#include <task.h>
#include <timers.h>
#include <mpu_prototypes.h>

#include <portable.h>

// FreeRTOS portmacro:
#include <portmacro.h>

// FreeRTOS config:
#include <FreeRTOSConfig.h>

// Shim for FreeRTOS macros:
#include "macro-shim.h"

// Weakly linked fallback functions.
#include "fallbacks.h"
