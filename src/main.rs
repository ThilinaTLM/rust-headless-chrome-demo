use std::path::PathBuf;
use std::fs;
use std::sync::mpsc::sync_channel;

use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::browser::tab::ResponseHandler;

const PAGE_URL: &str = "https://cooperative-colossus-630.notion.site/use-atroche-rust-headless-chrome-to-extract-some-json-from-this-page-the-one-your-reading-right-now-8080b648bf954d5aa66b9a0635594553";
const CHROME_EXECUTABLE_PATH: &str = "/usr/bin/brave-browser";
const FETCH_REQUEST_PATH: &str = "api/v3/loadCachedPageChunk";
const OUTPUT_FILE_PATH: &str = "outputs/loadCachedPageChunk.json";

/**
* This function returns a response handler
*/
fn create_response_handler(callback: Box<dyn Fn(bool) + Sync + Send>) -> ResponseHandler {
    return Box::new(move |params, get_body| {
        if params.response.url.ends_with(FETCH_REQUEST_PATH) {
            if params.response.status == 200 {
                let body = get_body().unwrap();
                fs::write(PathBuf::from(OUTPUT_FILE_PATH), body.body)
                    .expect("Unable to write file");
                println!("Wrote file {}", OUTPUT_FILE_PATH);
                callback(true);
            } else {
                println!("Browser fetch request seems to be not succeeded: {}", params.response.status);
                callback(false);
            }
        }
    })
}

fn main() {
    let (tx, rx) = sync_channel(1);

    // Create a new browser instance.
    let browser = if CHROME_EXECUTABLE_PATH.is_empty() {
        Browser::default()
    } else {
        Browser::new(LaunchOptions{
            path: Some(PathBuf::from(CHROME_EXECUTABLE_PATH)),
            ..Default::default()
        })
    };
    let browser = browser.expect("Failed to open browser");

    // Wait for the browser to finish loading.
    let tab = browser.wait_for_initial_tab().expect("failed to wait for initial tab");

    // Register a response handler.
    tab.enable_response_handling(create_response_handler(Box::new(move |_| {
        tx.send(true).unwrap();
    })))
        .expect("Failed to register response handler");

    // Load a page.
    tab.navigate_to(PAGE_URL).expect("failed to navigate to the page");

    // Wait for the response handler to be called.
    loop {
        match rx.try_recv() {
            Ok(_) => {
                println!("Done");
                break;
            }
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}