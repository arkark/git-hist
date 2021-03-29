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
                Arg::new("help")
                    .long("help")
                    .short('h')
                    .about("Print help information"),
            )
            .arg(
                Arg::new("version")
                    .long("version")
                    .short('v')
                    .about("Print version information"),
            )
            .arg(
                Arg::new("file")
                    .about("Set a target file path")
                    .required(true),
            )
            .get_matches();

        let file_path = matches.value_of("file").unwrap();

        Args {
            file_path: String::from(file_path),
        }
    }
}
