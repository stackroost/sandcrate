fn main() {
    let result = sandcrate_backend::run_plugin("assets/plugins/plugin_hello.wasm");
    println!("{:?}", result);
}
