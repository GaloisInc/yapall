#ifdef __cplusplus
extern "C" {
#endif

#ifdef RUN
void assert_points_to_something(void *p, ...) {}
void assert_reachable(void) {}
#else
extern void assert_points_to_something(void *p, ...);
extern void assert_reachable(void);
#endif

#ifdef __cplusplus
}
#endif
