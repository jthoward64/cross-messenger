/*
typedef struct ValidationData
{
  // Buffer of 512 bytes
  char *data;
  size_t length;
} ValidationData;

void generate_validation_data_binding(ValidationData *val)
 */

use base64::{engine::general_purpose, Engine as _};
use tauri::InvokeError;

#[repr(C)]
pub struct ValidationData {
    data: *mut libc::c_char,
    length: libc::size_t,
}

#[link(name = "emulated")]
extern "C" {
    fn generate_validation_data_binding(val: *mut ValidationData);
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

pub fn generate_validation_data() -> Result<String, GenerateValidationDataError> {
    let mut val = ValidationData {
        data: std::ptr::null_mut(),
        length: 0,
    };
    unsafe {
        generate_validation_data_binding(&mut val);
    }
    if val.data.is_null() {
        return Err(GenerateValidationDataError::ReturnedNull);
    }
    if val.length == 0 {
        return Err(GenerateValidationDataError::ZeroLengthBytes);
    }
    let bytes = unsafe { std::slice::from_raw_parts(val.data as *const u8, val.length) };
    let result = general_purpose::STANDARD.encode(bytes);
    unsafe {
        libc::free(val.data as *mut libc::c_void);
    }
    Ok(result)
}
