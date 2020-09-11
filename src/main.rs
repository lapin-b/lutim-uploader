extern crate dirs;
extern crate clipboard;
extern crate notify_rust;
extern crate inotify;
extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate webbrowser;

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use clipboard::ClipboardProvider;
use inotify::{EventMask, WatchMask};
use notify_rust::{Notification, Timeout};
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use reqwest::blocking::{multipart, Client};

use crate::structs::Config;

mod structs;

fn get_configuration(config_path: &PathBuf) -> Option<Config> {
    println!("Looking for configuration");

    if !config_path.exists() {
        let default_config = structs::Config::default();
        let mut default_config_file = BufWriter::new(File::create(&config_path).unwrap());

        default_config_file.write(
            serde_json::to_string_pretty(&default_config)
                .unwrap()
                .as_ref()
        ).unwrap();

        return None;
    }

    println!("Reading configuration");

    // The file exists, we continue with reading it. Unwrap the result since it should exist
    let config_file_reader = BufReader::new(File::open(&config_path).unwrap());
    let configuration: Config = serde_json::from_reader(config_file_reader).expect("Can't read configuration file");

    return Some(configuration);
}

fn main() {
    // Check for config file in configuration directory
    let mut config_path = dirs::config_dir().expect("Couln't find any operation system specific configuration directory");
    config_path.push("lutim_uploader_config.json");

    let configuration = match get_configuration(&config_path) {
        Some(c) => c,
        None => {
            println!("Default config file has been put in {}", &config_path.to_str().unwrap());
            return;
        }
    };

    // Check if directory to watch exists
    let watch_path = PathBuf::from(&configuration.watch_path);

    if !watch_path.is_dir() {
        println!("Creating folder to watch");
        std::fs::create_dir_all(&watch_path).expect("Can't create the directory to watch for changes");
    }

    let mut buffer = [0u8; 4096];
    let mut inotify_handle = inotify::Inotify::init().expect("Can't initialize inotify driver");
    inotify_handle.add_watch(&watch_path, WatchMask::CLOSE_WRITE).expect(&format!("Can't add a watch on {}", &watch_path.to_str().unwrap()));

    let http_client = Client::new();

    println!("Listening for events");

    loop {
        let events = inotify_handle
            .read_events_blocking(&mut buffer)
            .expect("Can't read inotify events");

        for event in events {
            if !event.mask.contains(EventMask::CLOSE_WRITE) {
                continue;
            }

            match event.name {
                Some(base_filename) => {
                    // Getting the basename and absolute path of the file to send
                    let base_filename = PathBuf::from(base_filename);
                    let mut absolute_base_filename = PathBuf::from(&configuration.watch_path);
                    absolute_base_filename.push(&base_filename);

                    println!("Uploading {}", base_filename.to_str().unwrap());

                    // Generate a random basename
                    let uploaded_filename: String = thread_rng()
                        .sample_iter(Alphanumeric)
                        .take(30)
                        .collect();

                    // Prepare the multipart part which will have the file content
                    let file_body =
                        multipart::Part::bytes(
                            std::fs::read(absolute_base_filename).expect("Can't read image")
                        )
                        .file_name(uploaded_filename + base_filename.extension().unwrap().to_str().unwrap());

                    // Multipart assembly
                    let body = multipart::Form::new()
                        .text("format", "json")
                        .text("first-view", "0")
                        .text("delete-day", "30")
                        .text("crypt", "1")
                        .text("keep-exif", "0")
                        .part("file", file_body);

                    // Actual request
                    let response = http_client.post(&configuration.lutim_url)
                        .multipart(body)
                        .send();

                    // Prepare some parts of the to be shown notification
                    let mut notification = Notification::new();
                    notification.appname = "lutim-uploader".to_string();

                    // Handle different outcomes of the HTTP response
                    match response {
                        Ok(rsp) => {
                            let status = rsp.status();

                            // Everything is fine: construct URL, set clipboard and open web browser
                            if status.is_success() {
                                let lutim_response: serde_json::Value = serde_json::from_str(&rsp.text().unwrap()).unwrap();
                                let final_url = format!(
                                    "{}/{}.{}",
                                    &configuration.lutim_url,
                                    &lutim_response.get("msg").unwrap().get("short").unwrap().as_str().unwrap(),
                                    &lutim_response.get("msg").unwrap().get("ext").unwrap().as_str().unwrap()
                                );

                                let mut clipboard_ctx = clipboard::ClipboardContext::new().unwrap();
                                clipboard_ctx.set_contents(final_url.clone()).unwrap();
                                webbrowser::open(&final_url).unwrap();

                                notification.body = "Image sent. Link copied to the clipboard".to_string();

                            // Server got itself into trouble
                            } else if status.is_server_error() {
                                notification.body = format!("Server error: {}", status)
                            }
                        }

                        // Something really wrong happened
                        Err(err) => {
                            eprintln!("{:?}", err);
                            notification.body = format!("Other error: {:?}", err)
                        }
                    };

                    notification.timeout = Timeout::Milliseconds(5000);
                    match notification.show() { _ => {} }
                }

                _ => {}
            }
        }
    }
}
