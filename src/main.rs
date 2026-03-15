use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1] == "ui" {
        tugas_block_cipher_rust::gpui_app::run_gpui();
        return;
    }

    process::exit(tugas_block_cipher_rust::run_cli());
}
