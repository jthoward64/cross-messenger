#define MACRO_PLIST_PATH "emulated/data.plist"
#define MACRO_BINARY_PATH "IMDAppleServices"

#include <Python.h>
#include "nac.c"
#include <stdio.h>

typedef struct ValidationData
{
  char *data;
  size_t length;
} ValidationData;

void generate_validation_data_binding(ValidationData *validationDataPtr)
{
  PyGILState_STATE state = PyGILState_Ensure();
  // Set the working directory to the directory of the executable
  // so that the Python interpreter can find the data files
  Py_SetPath(L"src-tauri/src/emulated");

  PyObject *validationData = __pyx_pf_3nac_68generate_validation_data(NULL);

  if (validationData != NULL && PyBytes_Check(validationData) && !PyErr_Occurred())
  {
    size_t length = PyBytes_Size(validationData);
    validationDataPtr->data = calloc(length, sizeof(char));

    validationDataPtr->length = length;
    memcpy(validationDataPtr->data, PyBytes_AsString(validationData), length);
  }
  else
  {
    printf("Error generating validation data\n");
  }

  Py_DECREF(validationData);
  PyGILState_Release(state);
}
