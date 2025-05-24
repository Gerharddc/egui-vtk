#include <vtk_glad.h>
#include <vtkActor.h>
#include <vtkCallbackCommand.h>
#include <vtkCamera.h>
#include <vtkCubeSource.h>
#include <vtkLogger.h>
#include <vtkNew.h>
#include <vtkPolyDataMapper.h>
#include <vtkProperty.h>
#include <vtkGenericOpenGLRenderWindow.h>
#include <vtkRenderer.h>
#include <vtkGenericRenderWindowInteractor.h>
#include <vtkInteractorStyleTrackballCamera.h>

#include "SimpleRenderer.h"

namespace
{
    vtkGenericOpenGLRenderWindow *render_window = nullptr;
    vtkGenericRenderWindowInteractor *interactor = nullptr;

    bool is_dirty = true;
    bool primary_down = false;
    bool secondary_down = false;
    bool middle_down = false;
    int window_width, window_height;

    void IsCurrentCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                           void *vtkNotUsed(clientData), void *callData)
    {
        // We always make sure to have the context for VTK active before calling render on it
        *(static_cast<bool *>(callData)) = true;
    }

    void FrameCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                       void *vtkNotUsed(clientData), void *vtkNotUsed(callData))
    {
        // FIXME: we should also actively send a redraw request to egui
        is_dirty = true;
    }
} // end anon namespace

void vtk_new(LoaderFunc load, int width, int height)
{
    vtkLogScopeFunction(INFO);
    vtkLogScopeF(INFO, "do-initialize");

    gladLoadGL(load);

    window_width = width;
    window_height = height;
    render_window = vtkGenericOpenGLRenderWindow::New();

    vtkNew<vtkRenderer> renderer;
    render_window->AddRenderer(renderer);
    render_window->SetSize(width, height);

    interactor = vtkGenericRenderWindowInteractor::New();
    interactor->SetRenderWindow(render_window);

    vtkNew<vtkInteractorStyleTrackballCamera> style;
    interactor->SetInteractorStyle(style);

    vtkNew<vtkCallbackCommand> is_current_cb;
    is_current_cb->SetCallback(IsCurrentCallback);
    render_window->AddObserver(vtkCommand::WindowIsCurrentEvent, is_current_cb);

    vtkNew<vtkCallbackCommand> frame_cb;
    frame_cb->SetCallback(FrameCallback);
    render_window->AddObserver(vtkCommand::WindowFrameEvent, frame_cb);

    vtkNew<vtkPolyDataMapper> mapper;
    vtkNew<vtkActor> actor;
    actor->SetMapper(mapper);
    renderer->AddActor(actor);
    vtkNew<vtkCubeSource> cs;
    mapper->SetInputConnection(cs->GetOutputPort());
    actor->RotateX(45.0);
    actor->RotateY(45.0);
    actor->GetProperty()->SetColor(0.8, 0.2, 0.2);
    renderer->ResetCamera();
    renderer->SetAutomaticLightCreation(true);
}

void vtk_destroy()
{
    assert(interactor != nullptr);
    assert(render_window != nullptr);

    interactor->Delete();
    render_window->Delete();

    interactor = nullptr;
    render_window = nullptr;
}

void vtk_paint()
{
    vtkLogScopeFunction(INFO);
    vtkLogScopeF(INFO, "do-vtk-render");

    assert(render_window != nullptr);
    render_window->Render();
    is_dirty = false;
}

bool vtk_is_dirty()
{
    return is_dirty;
}

void vtk_mouse_move(int x, int y)
{
    assert(interactor != nullptr);
    interactor->SetEventPosition(x, y);
    interactor->InvokeEvent(vtkCommand::MouseMoveEvent);
}

void vtk_update_mouse_down(bool primary, bool secondary, bool middle)
{
    assert(interactor != nullptr);

    if (primary_down != primary)
    {
        if (primary)
            interactor->InvokeEvent(vtkCommand::LeftButtonPressEvent);
        else
            interactor->InvokeEvent(vtkCommand::LeftButtonReleaseEvent);

        primary_down = primary;
    }

    if (secondary_down != secondary)
    {
        if (secondary)
            interactor->InvokeEvent(vtkCommand::RightButtonPressEvent);
        else
            interactor->InvokeEvent(vtkCommand::RightButtonReleaseEvent);

        secondary_down = secondary;
    }

    if (middle_down != middle)
    {
        if (middle)
            interactor->InvokeEvent(vtkCommand::MiddleButtonPressEvent);
        else
            interactor->InvokeEvent(vtkCommand::MiddleButtonReleaseEvent);

        middle_down = middle;
    }
}

void vtk_mouse_wheel(int delta)
{
    assert(interactor != nullptr);

    if (delta > 0)
    {
        interactor->InvokeEvent(vtkCommand::MouseWheelForwardEvent);
    }
    else if (delta < 0)
    {
        interactor->InvokeEvent(vtkCommand::MouseWheelBackwardEvent);
    }
}

void vtk_set_size(int width, int height)
{
    assert(render_window != nullptr);

    window_width = width;
    window_height = height;
    render_window->SetSize(width, height);

    if (interactor)
    {
        interactor->SetSize(width, height);
    }
}