use std::path::PathBuf;

fn main() -> iced::Result {
    let cli_paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    aetheris::app::run(cli_paths)
}
