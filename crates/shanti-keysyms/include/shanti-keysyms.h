/*
 * C ABI for the shanti-keysyms crate: the single-source keysym -> riti
 * virtual-key mapping used by the IBus engine.
 * Keysym values use the common X keysym space.
 * Unmapped keysyms resolve to VC_UNKNOWN (0x0046), the engine's historical
 * fallback.
 */
#ifndef SHANTI_KEYSYMS_H
#define SHANTI_KEYSYMS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

uint16_t shanti_keysyms_lookup(uint32_t keysym);

#ifdef __cplusplus
}
#endif

#endif /* SHANTI_KEYSYMS_H */
