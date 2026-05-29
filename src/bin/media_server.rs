/*
 * Copyright 2026 RidgeRun, LLC (http://www.ridgerun.com)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 * 1. Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright
 * notice, this list of conditions and the following disclaimer in the
 * documentation and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use clap::Parser;
use media_server::app::MediaServer;

#[derive(Debug, Parser)]
#[command(
    name = "media_server",
    about = "Rust media server",
    long_about = None
)]
struct Cli {
    /// Path to the main media server YAML configuration
    #[arg(short = 'c', long = "config", default_value = "cfg/media_server.yaml")]
    config_path: PathBuf,

    /// Path to the application log file
    #[arg(short = 'l', long = "log-file", default_value = "media_server.log")]
    log_file_path: PathBuf,

    /// Log level: 0=None, 1=Error, 2=Warning, 3=Info, 4=Debug
    #[arg(short = 'v', long = "log-level", default_value_t = 4)]
    log_level: i32,

    /// Append to log file (true) or overwrite it on startup (false)
    #[arg(long = "log-append", default_value_t = true, action = clap::ArgAction::Set)]
    log_append: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut media_server = match MediaServer::new(
        cli.config_path,
        cli.log_file_path,
        cli.log_level,
        cli.log_append,
    ) {
        Ok(media_server) => media_server,
        Err(err) => {
            eprintln!("failed to create media server: {:?}", err);
            return;
        }
    };

    if let Err(err) = media_server.start() {
        eprintln!("failed to start media server: {:?}", err);
        return;
    }

    println!("media server started");

    let running = Arc::new(AtomicBool::new(true));
    let signal_flag = Arc::clone(&running);

    ctrlc::set_handler(move || {
        signal_flag.store(false, Ordering::SeqCst);
    })
    .expect("failed to install Ctrl+C handler");

    println!("press Ctrl+C to stop the media server");

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(200));
    }

    println!("shutting down media server...");

    if let Err(err) = media_server.stop() {
        eprintln!("failed to stop media server cleanly: {:?}", err);
    }
}
