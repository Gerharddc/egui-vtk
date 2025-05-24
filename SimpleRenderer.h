#ifdef __cplusplus
extern "C"
{
#endif
    typedef void (*FuncPtr)(void);
    typedef FuncPtr (*LoaderFunc)(const char *name);

    void vtk_new(LoaderFunc load, int width, int height);
    void vtk_destroy();
    void vtk_paint();
    bool vtk_is_dirty();

#ifdef __cplusplus
}
#endif