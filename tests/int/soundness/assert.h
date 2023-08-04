#ifdef __cplusplus
#include <cstdint>
extern "C" {
#else
#include <stdint.h>
#endif

#ifdef RUN
void assert_constant(uint64_t, ...) {}
#else
extern void assert_constant(uint64_t, ...);
#endif

#ifdef __cplusplus
}
#endif
