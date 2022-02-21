/*
 * Copyright 2020 Skyscanner Limited.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
*/

use human_panic::setup_panic;
use log;
use protovend::git_url::GitUrl;
use structopt::clap::ArgGroup;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(group = ArgGroup::with_name("level"))]
struct Protovend {
    ///Prints only warnings and errors.
    #[structopt(long, group = "level")]
    warning: bool,
    ///Prints debug logs. Used for diagnostics only.
    #[structopt(long, group = "level")]
    debug: bool,
    #[structopt(subcommand)]
    sub: Subcommand,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Subcommand {
    ///Initialise current directory with protovend metadata file
    Init {},
    ///Add a given git repo to projects metadata file
    Add {
        url: GitUrl,
        #[structopt(short, long, default_value = "main")]
        branch: String,
    },
    ///Update one or all repos in protovend metadata file to latest version
    Update { repo: Option<GitUrl> },
    ///Install copies of protofiles declared in projects metadata file
    Install {},
    ///Delete all locally cached repos stored in protovend folder
    Cleanup {},
    ///Lint function to ensure proto files and directories are valid for the protovend tool
    Lint {},
}

fn setup_logger(level: log::LevelFilter) -> std::result::Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("({}) {}", record.level(), message)))
        .level(level)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn run_command(opts: Protovend) -> protovend::Result<()> {
    let level = if opts.debug {
        log::LevelFilter::Debug
    } else if opts.warning {
        log::LevelFilter::Warn
    } else {
        log::LevelFilter::Info
    };

    setup_logger(level)?;

    match opts.sub {
        Subcommand::Init {} => protovend::init(),
        Subcommand::Add { url, branch } => protovend::add(url, branch),
        Subcommand::Update { repo } => protovend::update(repo),
        Subcommand::Install {} => protovend::install(),
        Subcommand::Cleanup {} => protovend::cleanup(),
        Subcommand::Lint {} => protovend::lint(),
    }
}

fn main() {
    setup_panic!();

    std::process::exit(match run_command(Protovend::from_args()) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("Exiting early: {}", error);
            1
        }
    });
}
