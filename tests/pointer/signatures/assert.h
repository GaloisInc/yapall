#ifdef __cplusplus
extern "C" {
#endif

#ifdef RUN
void assert_may_alias(void *p, void *q) {}
void assert_points_to_something(void *p, ...) {}
#else
extern void assert_may_alias(void *p, void *q);
extern void assert_points_to_something(void *p, ...);
#endif

#ifdef __cplusplus
}
#endif
