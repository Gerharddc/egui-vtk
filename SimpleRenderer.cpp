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

    ExternalVTKWidget *externalVTKWidget = nullptr;
    bool initialized = false;
    int NumArgs;
    char **ArgV;
    bool tested = false;
    int retVal = 0;
    int windowId = -1;

    void MakeCurrentCallback(vtkObject *vtkNotUsed(caller), long unsigned int vtkNotUsed(eventId),
                             void *vtkNotUsed(clientData), void *vtkNotUsed(callData))
    {
        // std::cout << "TODO: make current\n";
    }

    void display()
    {
        vtkLogScopeFunction(INFO);
        if (!initialized)
        {
            vtkLogScopeF(INFO, "do-initialize");

            auto renWin = externalVTKWidget->GetRenderWindow();
            assert(renWin != nullptr);

            renWin->AutomaticWindowPositionAndResizeOff();
            renWin->SetSize(300, 300);
            renWin->SetPosition(300, 300);

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

    void onexit()
    {
        initialized = false;
    }

} // end anon namespace

void vtk_new(LoaderFunc load)
{
    gladLoadGL(load);
    externalVTKWidget = ExternalVTKWidget::New();
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