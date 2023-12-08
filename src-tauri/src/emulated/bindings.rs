/*
typedef struct ValidationData
{
  char *data;
  size_t length;
  bool is_valid;
} ValidationData;

ValidationData *generate_validation_data_binding()
 */

use std::ffi::c_char;

use tauri::InvokeError;

#[repr(C)]
#[derive(Debug)]
pub struct ValidationData {
    data: *mut c_char,
    length: usize,
}

#[link(name = "emulated")]
extern "C" {
    fn generate_validation_data_binding() -> ValidationData;
}

#[derive(Debug)]
pub enum GenerateValidationDataError {
    GenericError,
    CError,
    ReturnedNull,
    ZeroLengthBytes,
}

impl std::fmt::Display for GenerateValidationDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GenerateValidationDataError::GenericError => {
                write!(f, "Generic generate_validation_data() error")
            }
            GenerateValidationDataError::CError => {
                write!(f, "C error in generate_validation_data()")
            }
            GenerateValidationDataError::ReturnedNull => {
                write!(
                    f,
                    "Returned null (or similar) from generate_validation_data()"
                )
            }
            GenerateValidationDataError::ZeroLengthBytes => {
                write!(
                    f,
                    "Returned a bytes object with length 0 from generate_validation_data()"
                )
            }
        }
    }
}

impl std::error::Error for GenerateValidationDataError {}

impl Into<InvokeError> for GenerateValidationDataError {
    fn into(self) -> InvokeError {
        InvokeError::from(format!("{}", self))
    }
}

pub fn generate_validation_data() -> Result<Vec<c_char>, GenerateValidationDataError> {
    let validation_data = unsafe { generate_validation_data_binding() };
    if validation_data.data.is_null() {
        return Err(GenerateValidationDataError::ReturnedNull);
    }
    if validation_data.length == 0 {
        return Err(GenerateValidationDataError::ZeroLengthBytes);
    }
    let data = unsafe { std::slice::from_raw_parts(validation_data.data, validation_data.length) };
    Ok(data.to_vec())
}
