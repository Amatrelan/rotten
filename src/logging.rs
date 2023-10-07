use simplelog::TermLogger;

pub fn set_logger(level: log::LevelFilter) {
    let _ = TermLogger::init(
        level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    );
}

