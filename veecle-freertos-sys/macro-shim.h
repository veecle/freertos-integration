// Shim to wrap macros in functions to make them usable in Rust via bindgen.
// Bindgen doesn't support macros, which makes this wrapper necessary.

// FreeRTOS kernel includes:
#include <FreeRTOS.h>
//#include <atomic.h>
#include <croutine.h>
#include <FreeRTOS.h>
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

BaseType_t shim_pdFALSE(){
    return pdFALSE;
}

BaseType_t shim_pdTRUE(){
    return pdTRUE;
}

// Only exposed for testing, should not be used from rust code.
void shim_configASSERT(int value) {
    configASSERT(value);
}

char *shim_pcTaskGetName(TaskHandle_t xTaskToQuery) {
    return pcTaskGetName(xTaskToQuery);
}

TickType_t shim_portTICK_PERIOD_MS(){
    return portTICK_PERIOD_MS;
}

TickType_t shim_portMAX_DELAY(){
    return portMAX_DELAY;
}

void shim_taskYIELD(){
    taskYIELD();
}

BaseType_t shim_xQueueSendToBackFromISR
(
    QueueHandle_t xQueue,
    const void *pvItemToQueue,
    BaseType_t *pxHigherPriorityTaskWoken
){
    return xQueueSendToBackFromISR(xQueue, pvItemToQueue, pxHigherPriorityTaskWoken);
}

BaseType_t shim_xQueueSendToBack
(
    QueueHandle_t xQueue,
    const void *pvItemToQueue,
    TickType_t xTicksToWait
){
    return xQueueSendToBack(xQueue, pvItemToQueue, xTicksToWait);
}


QueueHandle_t shim_xQueueCreate
(
    UBaseType_t uxQueueLength,
    UBaseType_t uxItemSize
){
    return xQueueCreate(uxQueueLength, uxItemSize);
}

BaseType_t shim_xQueueReceive
(
    QueueHandle_t xQueue,
    void *pvBuffer,
    TickType_t xTicksToWait
){
    return xQueueReceive(xQueue, pvBuffer, xTicksToWait);
}

BaseType_t shim_xTaskNotify
(
    TaskHandle_t xTaskToNotify,
    uint32_t ulValue,
    eNotifyAction eAction
){
    return xTaskNotify(xTaskToNotify, ulValue, eAction);
}
BaseType_t shim_xTaskNotifyFromISR
(
    TaskHandle_t xTaskToNotify,
    uint32_t ulValue,
    eNotifyAction eAction,
    BaseType_t *pxHigherPriorityTaskWoken
){
    return xTaskNotifyFromISR(xTaskToNotify, ulValue, eAction, pxHigherPriorityTaskWoken);
}

BaseType_t shim_xTaskNotifyWait
(
    uint32_t ulBitsToClearOnEntry,
    uint32_t ulBitsToClearOnExit,
    uint32_t *pulNotificationValue,
    TickType_t xTicksToWait
){
    return xTaskNotifyWait(ulBitsToClearOnEntry, ulBitsToClearOnExit, pulNotificationValue, xTicksToWait);
}

uint32_t shim_ulTaskNotifyTake
(
    BaseType_t xClearCountOnExit,
    TickType_t xTicksToWait
){
    return ulTaskNotifyTake(xClearCountOnExit, xTicksToWait);
}

BaseType_t shim_xTimerStart
(
    TimerHandle_t xTimer,
    TickType_t xBlockTime
){
    return xTimerStart(xTimer, xBlockTime);
}

BaseType_t shim_xTimerStartFromISR
(
    TimerHandle_t xTimer,
    BaseType_t *pxHigherPriorityTaskWoken
){
    return xTimerStartFromISR(xTimer, pxHigherPriorityTaskWoken);
}

BaseType_t shim_xTimerStop
(
    TimerHandle_t xTimer,
    TickType_t xBlockTime
){
    return xTimerStop(xTimer, xBlockTime);
}

BaseType_t shim_xTimerChangePeriod
(
    TimerHandle_t xTimer,
    TickType_t xNewPeriod,
    TickType_t xBlockTime
){
    return xTimerChangePeriod(xTimer, xNewPeriod, xBlockTime);
}

BaseType_t shim_xTimerDelete
(
    TimerHandle_t xTimer,
    TickType_t xBlockTime
){
    return xTimerDelete(xTimer, xBlockTime);
}
