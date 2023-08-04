#ifdef __cplusplus
extern "C" {
#endif

#ifdef RUN
void assert_may_alias(void *p, void *q) {}
#else
extern void assert_may_alias(void *p, void *q);
#endif

#ifdef __cplusplus
}
#endif
