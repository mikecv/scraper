use log4rs;
use std::fs::File;
use std::io::{Write};

// Function to set up default logging if log settings file
// is not available or invalid.
pub fn set_up_logging() {
    // Attempt to open logging file.
    match log4rs::init_file("log4rs.yml", Default::default()) {
        Ok(_) => {},

        // Log settings not found or invalid, so
        // create default log4rs.yml file.
        Err(_) => {
            // Create default log4rs.yml content.
            // Formatting is important so don't mess with it.
            let default_log_config = "\
appenders:
  log_file:
    kind: rolling_file
    append: true
    path: \"logs/scraper.log\"
    encoder:
      pattern: \"{h({d(%d-%m-%Y %H:%M:%S)})} - {l:<5} {t}:{L} - {m}{n}\"
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 10mb
      roller:
        kind: fixed_window
        base: 1
        count: 3
        pattern: \"logs/scraper{}.log\"

root:
  level: debug
  appenders:
    - log_file
";
            // Try to create the log4rs.yml file.
            if let Ok(mut file) = File::create("log4rs.yml") {
                let _ = file.write_all(default_log_config.as_bytes());
            }

            // Now try to initialize logging again with the newly created file.
            match log4rs::init_file("log4rs.yml", Default::default()) {
                Ok(_) => {},
                // If it still fails, fall back to console logging
                Err(_) => {
                    use log4rs::append::console::ConsoleAppender;
                    use log4rs::encode::pattern::PatternEncoder;
                    use log4rs::config::{Appender, Config, Root};
                    use log::LevelFilter;

                    let stdout = ConsoleAppender::builder()
                        .encoder(Box::new(PatternEncoder::new(
                            "{h({d(%H:%M:%S)})} - {m}{n}"
                        )))
                        .build();

                    let config = Config::builder()
                        .appender(Appender::builder().build("stdout", Box::new(stdout)))
                        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
                        .unwrap();

                    log4rs::init_config(config).unwrap();
                }
            }
        }
    }
}
