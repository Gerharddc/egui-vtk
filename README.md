# Building VTK

    cmake -GNinja .. -DVTK_MODULE_ENABLE_VTK_RenderingExternal:String=YES -DVTK_SMP_IMPLEMENTATION_TYPE=STDThread -DBUILD_SHARED_LIBS=OFF
    cmake --build . --config Release
    cmake --install . --prefix C:\Users\gerha\Projects\vtk-msvc-static