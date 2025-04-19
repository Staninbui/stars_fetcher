# Stars Fetcher

Stars Fetcher 是一个用 Rust 编写的命令行工具，用于获取 GitHub 仓库的 star 数量。


## 特性

- 获取指定 GitHub 仓库的 star 数量
- 支持多个仓库的批量查询
- 输出结果到终端

## 安装

首先，确保你已经安装了 Rust 和 Cargo。然后运行以下命令来安装 Stars Fetcher：

```sh
cargo install stars_fetcher
```

## 使用方法

### 获取单个仓库的 star 数量

```sh
stars_fetcher <owner>/<repo>
```

例如：

```sh
stars_fetcher rust-lang/rust
```

### 批量获取多个仓库的 star 数量

你可以在一个文件中列出多个仓库，每行一个，然后使用以下命令：

```sh
stars_fetcher -f <file_path>
```

例如，假设你有一个名为 `repos.txt` 的文件，内容如下：

```
rust-lang/rust
tokio-rs/tokio
serde-rs/serde
```

你可以运行以下命令来获取这些仓库的 star 数量：

```sh
stars_fetcher -f repos.txt
```