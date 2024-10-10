use prost::Message;
use std::os::raw::c_void;
use std::slice;

pub mod ffi {
    pub mod messaging {
        include!(concat!(env!("OUT_DIR"), "/ffi.messaging.rs"));
    }
}

use ffi::messaging::{FromRust, ToRust};

// Type for the callback function
type CallbackFn = extern "C" fn(*const u8, usize, *mut c_void);

#[repr(C)]
pub struct MessageFatPointer {
    pub data_pointer: *const u8,
    pub len: usize,
}
//when data_pointer is 0, len describes the error
//len 1 = Failed to parse input Protocol Buffer
#[no_mangle]
pub extern "C" fn sync_function(args: MessageFatPointer) -> MessageFatPointer {
    let input_slice = unsafe { slice::from_raw_parts(args.data_pointer, args.len) };
    let decode = ToRust::decode(input_slice);
    let request = match decode {
        Ok(req) => req,
        Err(_) => {
            eprintln!("Failed to parse input Protocol Buffer");
            return MessageFatPointer {
                data_pointer: std::ptr::null(),
                len: 1,
            };
        }
    };
    let response: FromRust = process_request(request);

    let response_bytes = response.encode_to_vec();

    // Call the callback with the serialized response
    return MessageFatPointer {
        data_pointer: response_bytes.as_ptr(),
        len: response_bytes.len(),
    };
}

#[no_mangle]
pub extern "C" fn process_proto(
    data: *const u8,
    len: usize,
    callback: CallbackFn,
    user_data: *mut c_void,
) {
    // Convert the raw pointer to a slice
    let input_slice = unsafe { slice::from_raw_parts(data, len) };

    // Deserialize the input Protocol Buffer
    let dec = ToRust::decode(input_slice);
    let request = match dec {
        Ok(req) => req,
        Err(_) => {
            eprintln!("Failed to parse input Protocol Buffer");
            return;
        }
    };

    // Process the request
    let response: FromRust = process_request(request);

    let response_bytes = response.encode_to_vec();

    // Call the callback with the serialized response
    callback(response_bytes.as_ptr(), response_bytes.len(), user_data);
}

fn process_request(request: ToRust) -> FromRust {
    // Implement your processing logic here
    // This is just a placeholder
    let req = dbg!(request);
    let mut retval = FromRust::default();

    if let Some(add_req) = req.request_add {
        retval.add_result = add_req.v1 + add_req.v2;
    }

    retval
}
