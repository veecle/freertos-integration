#include <FreeRTOS.h>

// Fallback implementation of `vPortGetHeapStats` for heap implementations that don't support it.
void __attribute__((weak)) vPortGetHeapStats(__attribute__((unused)) HeapStats_t * pxHeapStats ) {
    // Do nothing.
}
