# Building VTK

    cmake -GNinja .. -DVTK_SMP_IMPLEMENTATION_TYPE=STDThread -DBUILD_SHARED_LIBS=OFF
    cmake --build . --config Release
    cmake --install . --prefix C:\Users\gerha\Projects\vtk-msvc-static