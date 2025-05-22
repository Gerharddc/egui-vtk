cmake_minimum_required(VERSION 3.28)
project(vtktest)

include(FetchContent)

if(COMMAND cmake_policy)
    cmake_policy(SET CMP0003 NEW)
endif(COMMAND cmake_policy)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5 # Optionally specify a commit hash, version tag or branch here
)
FetchContent_MakeAvailable(Corrosion)

find_package(VTK
 COMPONENTS
    CommonCore
    FiltersSources
    RenderingCore
    RenderingExternal
    TestingRendering
    InteractionStyle  # implements VTK::RenderingCore
    RenderingCellGrid # implements VTK::RenderingCore
    RenderingFreeType # implements VTK::RenderingCore
    RenderingOpenGL2  # implements VTK::RenderingCore
    RenderingUI       # implements VTK::RenderingCore
)

add_executable(vtktest SimpleRenderer.cpp)

target_link_libraries(vtktest PRIVATE ${VTK_LIBRARIES})

# Windows libraries for Rust
target_link_libraries(vtktest PRIVATE uxtheme Imm32 Dwmapi uiautomationcore oleaut32 propsys RuntimeObject Shlwapi)

corrosion_import_crate(MANIFEST_PATH rust-lib/Cargo.toml)
target_link_libraries(vtktest PRIVATE vtk_rs)

# vtk_module_autoinit is needed
vtk_module_autoinit(
    TARGETS vtktest
    MODULES ${VTK_LIBRARIES}
)
