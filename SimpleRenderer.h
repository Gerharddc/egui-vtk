#ifdef __cplusplus
extern "C"
{
#endif
    typedef void (*FuncPtr)(void);
    typedef FuncPtr (*LoaderFunc)(const char *name);

    void vtk_new(LoaderFunc load);
    void vtk_destroy();
    void vtk_paint();

#ifdef __cplusplus
}
#endif