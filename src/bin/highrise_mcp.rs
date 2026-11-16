fn main() {
    if let Err(err) = highrise_rs::mcp::run_stdio_server() {
        eprintln!("highrise_mcp error: {err}");
        std::process::exit(1);
    }
}
