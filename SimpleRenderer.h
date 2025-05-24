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

    void vtk_mouse_move(int x, int y);
    void vtk_mouse_press(int button, int x, int y);
    void vtk_mouse_release(int button, int x, int y);
    void vtk_mouse_wheel(int delta, int x, int y);
    void vtk_set_size(int width, int height);

#ifdef __cplusplus
}
#endif