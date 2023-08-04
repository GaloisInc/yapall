#ifdef __cplusplus
extern "C" {
#endif

#ifdef RUN
void assert_disjoint(void *p, void *q) {}
void assert_points_to_nothing(void *p, ...) {}
void assert_unreachable(void) {}
#else
extern void assert_disjoint(void *p, void *q);
extern void assert_points_to_nothing(void *p, ...);
extern void assert_unreachable(void);
#endif

#ifdef __cplusplus
}
#endif
