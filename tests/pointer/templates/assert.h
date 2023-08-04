#ifdef __cplusplus
extern "C" {
#endif

#ifdef RUN
void assert_points_to_something(void *p, ...) {}
#else
extern void assert_points_to_something(void *p, ...);
#endif

#ifdef __cplusplus
}
#endif
