use clap::{App, Arg};

#[derive(Debug)]
pub struct Args {
    pub file_path: String,
    pub should_use_full_commit_hash: bool,
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
                Arg::new("full-hash")
                    .long("full-hash")
                    .about("Show full commit hashes instead of abbreviated commit hashes"),
            )
            .arg(
                Arg::new("file")
                    .about("Set a target file path")
                    .required(true),
            )
            .get_matches();

        let file_path = matches.value_of("file").unwrap();
        let should_use_full_commit_hash = matches.is_present("full-hash");
        dbg!(should_use_full_commit_hash);

        Args {
            file_path: String::from(file_path),
            should_use_full_commit_hash,
        }
    }
}
