use clap::{App, Arg};

#[derive(Debug)]
pub struct Args {
    pub file_path: String,
    pub should_use_full_commit_hash: bool,
    pub beyond_last_line: bool,
    pub user_for_name: UserType,
    pub user_for_date: UserType,
}

#[derive(Debug)]
pub enum UserType {
    Author,
    Committer,
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
                Arg::new("beyond-last-line")
                    .long("beyond-last-line")
                    .about("Set whether the view will scroll beyond the last line"),
            )
            .arg(
                Arg::new("name-of")
                    .long("name-of")
                    .value_name("user")
                    .possible_values(&["author", "committer"])
                    .default_value("author")
                    .about("Use whether authors or committers for names"),
            )
            .arg(
                Arg::new("date-of")
                    .long("date-of")
                    .value_name("user")
                    .possible_values(&["author", "committer"])
                    .default_value("author")
                    .about("Use whether authors or committers for dates"),
            )
            .arg(
                Arg::new("file")
                    .about("Set a target file path")
                    .required(true),
            )
            .get_matches();

        let file_path = String::from(matches.value_of("file").unwrap());

        let should_use_full_commit_hash = matches.is_present("full-hash");
        let beyond_last_line = matches.is_present("beyond-last-line");
        let user_for_name = if matches.value_of("name-of").unwrap() == "author" {
            UserType::Author
        } else {
            UserType::Committer
        };
        let user_for_date = if matches.value_of("date-of").unwrap() == "author" {
            UserType::Author
        } else {
            UserType::Committer
        };

        Args {
            file_path,
            should_use_full_commit_hash,
            beyond_last_line,
            user_for_name,
            user_for_date,
        }
    }
}
