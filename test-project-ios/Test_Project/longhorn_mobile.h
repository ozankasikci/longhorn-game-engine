#ifndef longhorn_mobile_h
#define longhorn_mobile_h

#include <stdbool.h>
#include <stdint.h>

bool longhorn_init(void);
bool longhorn_create_with_metal(void* metal_layer, uint32_t width, uint32_t height);
bool longhorn_load_game(const char* path);
bool longhorn_start(void);
bool longhorn_update(void);
void longhorn_handle_touch_start(float x, float y);
void longhorn_handle_touch_move(float x, float y);
void longhorn_handle_touch_end(float x, float y);
void longhorn_resize(uint32_t width, uint32_t height);
void longhorn_cleanup(void);

#endif /* longhorn_mobile_h */
