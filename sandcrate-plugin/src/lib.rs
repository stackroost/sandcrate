use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn main() {
    println!("Hello from WASM plugin!");
    println!("Plugin execution started");
    
    // Try to get parameters if available
    if let Some(params) = get_parameters() {
        println!("Received parameters: {}", params);
    } else {
        println!("No parameters provided");
    }
    
    // Simulate some work
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
    // This would be called by the host environment
    // For now, we'll return None to indicate no parameters
    None
}

// Export a function that can be called from the host
#[no_mangle]
pub extern "C" fn process_data(input: *const c_char) -> *const c_char {
    if input.is_null() {
        return std::ptr::null();
    }
    
    let input_str = unsafe {
        CStr::from_ptr(input).to_string_lossy().into_owned()
    };
    
    let result = format!("Processed: {}", input_str);
    
    // In a real implementation, you'd need to manage memory properly
    // For this demo, we'll just return a static string
    "Data processed successfully\0".as_ptr() as *const c_char
}
