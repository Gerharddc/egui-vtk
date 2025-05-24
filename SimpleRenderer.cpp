#include <vtk_glad.h>
#include <ExternalVTKWidget.h>
#include <vtkActor.h>
#include <vtkCallbackCommand.h>
#include <vtkCamera.h>
#include <vtkCubeSource.h>
#include <vtkExternalOpenGLRenderWindow.h>
#include <vtkLight.h>
#include <vtkLogger.h>
#include <vtkNew.h>
#include <vtkPolyDataMapper.h>
#include <vtkTesting.h>
#include <vtkProperty.h>
#include <vtkGenericOpenGLRenderWindow.h>

#include "SimpleRenderer.h"

namespace
{
    vtkGenericOpenGLRenderWindow *render_window;
    bool initialized = false;

    void MakeCurrentCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                             void *vtkNotUsed(clientData), void *vtkNotUsed(callData))
    {
        // std::cout << "TODO: make current\n";
    }

    void IsCurrentCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                           void *vtkNotUsed(clientData), void *callData)
    {
        // std::cout << "TODO: is current\n";
        *(static_cast<bool *>(callData)) = true;
    }

    void FrameCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                       void *vtkNotUsed(clientData), void *vtkNotUsed(callData))
    {
        // std::cout << "TODO: frame\n";
    }

    void display()
    {
        vtkLogScopeFunction(INFO);
        if (!initialized)
        {
            vtkLogScopeF(INFO, "do-initialize");

            vtkNew<vtkRenderer> renderer;
            render_window->AddRenderer(renderer);

            render_window->SetSize(300, 300);
            render_window->SetPosition(300, 300);

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

        vtkLogScopeF(INFO, "do-vtk-render");
        render_window->Render();
    }

    void onexit()
    {
        initialized = false;
    }

} // end anon namespace

void vtk_new(LoaderFunc load)
{
    gladLoadGL(load);
    render_window = vtkGenericOpenGLRenderWindow::New();
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