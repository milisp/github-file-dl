# github-file-dl

> download github special files. Resuming interrupted downloads  

> 下载 github 某个文件夹或文件，可以断点续传

[api](https://docs.github.com/en/rest/repos/contents) - github contents api


## python
### Install

```sh
pip install git+https://github.com/milisp/github-file-dl
```

### Usage

```sh
github-file-dl -h
github-file-dl github_url download_folder
github-file-dl github_url download_folder -p proxy_url
```

### main code file

[`__main__.py`](src/github_file_dl/__main__.py)

## Similar or related Projects

- [github-files-fetcher](https://github.com/Gyumeijie/github-files-fetcher) - nodejs
- [fetch](https://github.com/gruntwork-io/fetch) - golang
