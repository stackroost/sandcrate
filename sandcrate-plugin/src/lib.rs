use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn main() {
    println!("Plugin execution started");
    if let Some(params) = get_parameters() {
        println!("Received parameters: {}", params);
    } else {
        println!("No parameters provided");
    }
    for i in 1..=3 {
        println!("Processing step {}", i);
    }
    
    println!("Plugin execution completed successfully!");
}

#[no_mangle]
pub extern "C" fn run() {
    println!("Alternative run function called!");
    println!("This demonstrates multiple entry points");
}

fn get_parameters() -> Option<String> {
    None
}
#[no_mangle]
pub extern "C" fn process_data(input: *const c_char) -> *const c_char {
    if input.is_null() {
        return std::ptr::null();
    }
    
    let input_str = unsafe {
        CStr::from_ptr(input).to_string_lossy().into_owned()
    };
    
    let result = format!("Processed: {}", input_str);
    "Data processed successfully\0".as_ptr() as *const c_char
}
