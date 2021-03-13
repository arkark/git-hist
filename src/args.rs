use clap::{App, Arg};

#[derive(Debug)]
pub struct Args {
    pub file_path: String,
}

impl Args {
    pub fn load() -> Args {
        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .setting(clap::AppSettings::ColoredHelp)
            .arg(
                Arg::new("file")
                    .about("Sets a target file path")
                    .required(true),
            )
            .get_matches();

        let file_path = matches.value_of("file").unwrap();

        Args {
            file_path: String::from(file_path),
        }
    }
}
