#define MACRO_PLIST_PATH "src-tauri/src/emulated/data.plist"
#define MACRO_BINARY_PATH "src-tauri/src/emulated/IMDAppleServices"

#include "nac.c"

typedef struct ValidationData
{
  char *data;
  size_t length;
} ValidationData;

ValidationData generate_validation_data_binding()
{
  PyObject *validationData = __pyx_pf_3nac_68generate_validation_data(NULL);

  if (validationData == NULL)
  {
    return (ValidationData){NULL, 0};
  }

  if (!PyBytes_Check(validationData))
  {
    Py_DECREF(validationData);
    return (ValidationData){NULL, 0};
  }

  if (PyErr_Occurred())
  {
    Py_DECREF(validationData);
    return (ValidationData){NULL, 0};
  }

  size_t length = PyBytes_Size(validationData);

  char *data = calloc(length, sizeof(char));

  memcpy(data, PyBytes_AsString(validationData), length);

  Py_DECREF(validationData);

  return (ValidationData){data, length};
}
