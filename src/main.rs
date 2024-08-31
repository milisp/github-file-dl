use log::{error, info};
use reqwest::{blocking::Client, Proxy};
use serde::Deserialize;
use std::env;
use std::fs::{self, create_dir_all};
use url::Url;

#[derive(Deserialize)]
struct GitHubContent {
    name: String,
    path: String,
    r#type: String,
    download_url: Option<String>,
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        error!("Usage: <executable> <github-folder-url> [proxy-url]");
        return;
    }

    let url = &args[1];
    let proxy_url = args.get(2);

    match Url::parse(url) {
        Ok(parsed_url) => {
            let segments: Vec<&str> = parsed_url
                .path_segments()
                .map(|c| c.collect::<Vec<_>>())
                .unwrap_or_default();

            if segments.len() < 4 || segments[2] != "tree" {
                error!("Invalid GitHub folder URL.");
                return;
            }

            let owner = segments[0];
            let repo = segments[1];
            let branch = segments[3];
            let folder_path: String = segments[4..].join("/");

            let api_url = format!(
                "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                owner, repo, folder_path, branch
            );

            info!("API URL: {}\n", api_url); // Log the API URL for debugging
            print!("API URL: {}\n", api_url); // Log the API URL for debugging

            let mut client_builder = Client::builder().user_agent("github-folder-downloader");

            if let Some(proxy_url) = proxy_url {
                match Proxy::all(proxy_url) {
                    Ok(proxy) => {
                        client_builder = client_builder.proxy(proxy);
                        info!("Using proxy: {}\n", proxy_url);
                    }
                    Err(e) => {
                        error!("Invalid proxy URL: {}", e);
                        return;
                    }
                }
            }

            let client = client_builder.build().unwrap();

            match client.get(&api_url).send() {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<Vec<GitHubContent>>() {
                            Ok(contents) => {
                                download_folder_contents(
                                    &client,
                                    &contents,
                                    &folder_path,
                                    owner,
                                    repo,
                                    branch,
                                );
                            }
                            Err(e) => error!("Failed to parse JSON: {}", e),
                        }
                    } else {
                        error!("Failed to fetch folder contents: {}", response.status());
                        error!("Response text: {}", response.text().unwrap_or_default());
                        // Log the response text for debugging
                    }
                }
                Err(e) => error!("Failed to send request: {}", e),
            }
        }
        Err(e) => error!("Invalid URL: {}", e),
    }
}

fn download_folder_contents(
    client: &Client,
    contents: &[GitHubContent],
    folder_path: &str,
    owner: &str,
    repo: &str,
    branch: &str,
) {
    create_dir_all(folder_path).unwrap();

    for item in contents {
        let item_path = format!("{}/{}", folder_path, item.name);

        if item.r#type == "dir" {
            info!("Creating directory: {}", item_path);
            create_dir_all(&item_path).unwrap();

            let api_url = format!(
                "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                owner, repo, item.path, branch
            );

            match client.get(&api_url).send() {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<Vec<GitHubContent>>() {
                            Ok(contents) => download_folder_contents(
                                client, &contents, &item_path, owner, repo, branch,
                            ),
                            Err(e) => error!("Failed to parse JSON: {}", e),
                        }
                    } else {
                        error!("Failed to fetch folder contents: {}", response.status());
                        error!("Response text: {}", response.text().unwrap_or_default());
                        // Log the response text for debugging
                    }
                }
                Err(e) => error!("Failed to send request: {}", e),
            }
        } else if let Some(download_url) = &item.download_url {
            info!("Downloading file: {}", item_path);
            match client.get(download_url).send() {
                Ok(mut response) => {
                    if response.status().is_success() {
                        let mut file = fs::File::create(&item_path).unwrap();
                        response.copy_to(&mut file).unwrap();
                    } else {
                        error!("Failed to download file: {}", response.status());
                    }
                }
                Err(e) => error!("Failed to send request: {}", e),
            }
        }
    }
}
