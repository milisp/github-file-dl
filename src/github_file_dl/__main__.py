import argparse
import os
import requests
import logging
from urllib.parse import urlparse, parse_qs, urljoin

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def get_github_api_url(github_url):
    """
    Convert a GitHub URL into a GitHub API URL for accessing the repository contents.
    """
    parsed_url = urlparse(github_url)
    path_parts = parsed_url.path.strip("/").split("/")

    if len(path_parts) < 3 or path_parts[2] != "tree":
        raise ValueError(
            "Invalid GitHub URL. Please ensure it is a URL to a specific folder within the repository."
        )

    user, repo, _, branch, *folder_path = path_parts
    api_url = f"https://api.github.com/repos/{user}/{repo}/contents/{'/'.join(folder_path)}?ref={branch}"
    return api_url, branch, "/".join(folder_path)


def download_folder(api_url, local_dir, proxies=None):
    """
    Download a folder and its contents from the GitHub API.
    """
    logger.info(f"Fetching contents from {api_url}")
    response = requests.get(api_url, proxies=proxies)
    response.raise_for_status()

    items = response.json()

    if isinstance(items, dict) and items.get("message") == "Not Found":
        logger.error("Folder not found in the repository.")
        return

    for item in items:
        item_name = item["name"]
        item_path = os.path.join(local_dir, item_name)

        if item["type"] == "file":
            logger.info(f"Downloading file {item_name}")
            download_file(item["download_url"], item_path, proxies)
        elif item["type"] == "dir":
            logger.info(f"Creating directory {item_name}")
            os.makedirs(item_path, exist_ok=True)
            download_folder(item["url"], item_path, proxies)


def download_file(file_url, file_path, proxies=None):
    """
    Download a single file from the GitHub API.
    """
    logger.info(f"Downloading {file_url}")
    response = requests.get(file_url, proxies=proxies)
    response.raise_for_status()

    with open(file_path, "wb") as file:
        file.write(response.content)
    logger.info(f"Saved to {file_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Download a specific folder from a GitHub repository."
    )
    parser.add_argument(
        "github_url",
        help="GitHub URL to the folder (e.g., https://github.com/user/repo/tree/branch/folder)",
    )
    parser.add_argument(
        "output_dir", help="Local directory to save the downloaded contents"
    )
    parser.add_argument(
        "-p", "--proxy", help="Proxy URL (e.g., http://proxy.example.com:8080)", default=None
    )

    args = parser.parse_args()

    proxies = None
    print(args)
    if args.proxy:
        proxies = {
            "http": args.proxy,
            "https": args.proxy,
        }
        logger.info(f"Using proxy: {args.proxy}")

    try:
        api_url, branch, folder_path = get_github_api_url(args.github_url)
        os.makedirs(args.output_dir, exist_ok=True)
        download_folder(api_url, args.output_dir, proxies)
        logger.info(
            f"Successfully downloaded folder '{folder_path}' from branch '{branch}'"
        )
    except Exception as e:
        logger.error(f"Error: {e}")


if __name__ == "__main__":
    main()
