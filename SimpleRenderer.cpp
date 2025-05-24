#include <iostream>

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

#include "SimpleRenderer.h"

namespace
{

    // Global variables used by the glutDisplayFunc and glutIdleFunc
    ExternalVTKWidget *externalVTKWidget = nullptr;
    bool initialized = false;
    int NumArgs;
    char **ArgV;
    bool tested = false;
    int retVal = 0;
    int windowId = -1;
    int windowH = 301;
    int windowW = 300;

    void MakeCurrentCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                             void *vtkNotUsed(clientData), void *vtkNotUsed(callData))
    {
        vtkLogScopeFunction(1);
        if (initialized)
        {
            // TODO
            // glutSetWindow(windowId);
        }
    }

    /* Handler for window-repaint event. Call back when the window first appears and
       whenever the window needs to be re-painted. */
    void display()
    {
        vtkLogScopeFunction(INFO);
        if (!initialized)
        {
            vtkLogScopeF(INFO, "do-initialize");
            // since `handleResize` may get called before display, we may have already
            // created and resized the vtkExternalOpenGLRenderWindow, hence we don't
            // recreate it here.
            auto renWin = externalVTKWidget->GetRenderWindow();

            // since our example here is not setting up the `glViewport`, we don't want
            // the vtkExternalOpenGLRenderWindow to update its size based on the
            // glViewport hence we must disable automatic position and size.
            renWin->AutomaticWindowPositionAndResizeOff();

            assert(renWin != nullptr);
            vtkNew<vtkCallbackCommand> callback;
            callback->SetCallback(MakeCurrentCallback);
            renWin->AddObserver(vtkCommand::WindowMakeCurrentEvent, callback);
            vtkNew<vtkPolyDataMapper> mapper;
            vtkNew<vtkActor> actor;
            actor->SetMapper(mapper);
            vtkRenderer *ren = externalVTKWidget->AddRenderer();
            ren->AddActor(actor);
            vtkNew<vtkCubeSource> cs;
            mapper->SetInputConnection(cs->GetOutputPort());
            actor->RotateX(45.0);
            actor->RotateY(45.0);
            actor->GetProperty()->SetColor(0.8, 0.2, 0.2);
            ren->ResetCamera();
            ren->SetAutomaticLightCreation(true);

            initialized = true;
        }

        vtkLogScopeF(INFO, "do-vtk-render");
        externalVTKWidget->GetRenderWindow()->Render();
    }

    void handleResize(int w, int h)
    {
        vtkLogScopeF(INFO, "handleResize: %d, %d", w, h);
        externalVTKWidget->GetRenderWindow()->SetSize(w, h);
    }

    void onexit()
    {
        initialized = false;
    }

} // end anon namespace

void vtk_new(LoaderFunc load)
{
    gladLoadGL(load);
    // gladLoaderLoadGL();
    externalVTKWidget = ExternalVTKWidget::New();
    handleResize(300, 300);
}

void vtk_destroy()
{
    onexit();
    externalVTKWidget->Delete();
}

void vtk_paint()
{
    display();
}
