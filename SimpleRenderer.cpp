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

    bool initialized = false;
    bool dirty = true;
    int window_width, window_height;

    void MakeCurrentCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                             void *vtkNotUsed(clientData), void *vtkNotUsed(callData))
    {
        // TODO: perhaps we should actually do something here?
    }

    void IsCurrentCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                           void *vtkNotUsed(clientData), void *callData)
    {
        // TODO: perhaps we should actually do something here?
        *(static_cast<bool *>(callData)) = true;
    }

    void FrameCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                       void *vtkNotUsed(clientData), void *vtkNotUsed(callData))
    {
        // FIXME: we should also actively send a redraw request to egui
        dirty = true;
    }

    void init(int width, int height)
    {
        vtkLogScopeFunction(INFO);
        vtkLogScopeF(INFO, "do-initialize");

        window_width = width;
        window_height = height;

        vtkNew<vtkRenderer> renderer;
        render_window->AddRenderer(renderer);
        render_window->SetSize(width, height);

        interactor = vtkGenericRenderWindowInteractor::New();
        interactor->SetRenderWindow(render_window);

        vtkNew<vtkInteractorStyleTrackballCamera> style;
        interactor->SetInteractorStyle(style);

        vtkNew<vtkCallbackCommand> make_current_cb;
        make_current_cb->SetCallback(MakeCurrentCallback);
        render_window->AddObserver(vtkCommand::WindowMakeCurrentEvent, make_current_cb);

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

        initialized = true;
    }

    void display()
    {
        vtkLogScopeFunction(INFO);
        vtkLogScopeF(INFO, "do-vtk-render");

        assert(initialized);
        render_window->Render();
        dirty = false;
    }

    void onexit()
    {
        initialized = false;
    }

} // end anon namespace

void vtk_new(LoaderFunc load, int width, int height)
{
    gladLoadGL(load);
    render_window = vtkGenericOpenGLRenderWindow::New();
    init(width, height);
}

void vtk_destroy()
{
    onexit();
    render_window->Delete();
}

void vtk_paint()
{
    display();
}

bool vtk_is_dirty()
{
    return dirty;
}

void vtk_mouse_move(int x, int y)
{
    assert(interactor);

    interactor->SetEventPosition(x, y);
    interactor->InvokeEvent(vtkCommand::MouseMoveEvent);
}

void vtk_mouse_press(int button, int x, int y)
{
    assert(interactor);

    interactor->SetEventPosition(x, y);

    switch (button)
    {
    case 0: // Left button
        interactor->InvokeEvent(vtkCommand::LeftButtonPressEvent);
        break;
    case 1: // Right button
        interactor->InvokeEvent(vtkCommand::RightButtonPressEvent);
        break;
    case 2: // Middle button
        interactor->InvokeEvent(vtkCommand::MiddleButtonPressEvent);
        break;
    }
}

void vtk_mouse_release(int button, int x, int y)
{
    assert(interactor);

    interactor->SetEventPosition(x, y);

    switch (button)
    {
    case 0: // Left button
        interactor->InvokeEvent(vtkCommand::LeftButtonReleaseEvent);
        break;
    case 1: // Right button
        interactor->InvokeEvent(vtkCommand::RightButtonReleaseEvent);
        break;
    case 2: // Middle button
        interactor->InvokeEvent(vtkCommand::MiddleButtonReleaseEvent);
        break;
    }
}

void vtk_mouse_wheel(int delta)
{
    assert(interactor);

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
    assert(initialized);

    window_width = width;
    window_height = height;
    render_window->SetSize(width, height);

    if (interactor)
    {
        interactor->SetSize(width, height);
    }
}