#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--import-folder" {
        match baasic_media_player_lib::cli_import(&args[2]) {
            Ok(result) => {
                println!(
                    "BAASIC import complete: {} added, {} duplicates skipped, {} auto-tagged",
                    result.imported, result.skipped_duplicates, result.tagged
                );
                if !result.errors.is_empty() {
                    eprintln!("{} errors (see log)", result.errors.len());
                }
            }
            Err(e) => {
                eprintln!("Import failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    baasic_media_player_lib::run();
}
