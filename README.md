# Bookify-rs

[![Crates.io Version](https://img.shields.io/crates/v/bookify_rs)](https://crates.io/crates/bookify_rs)
[![docs.rs](https://img.shields.io/docsrs/bookify_rs)](https://docs.rs/bookify_rs)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/wesleyel/bookify_rs/release.yml)](https://github.com/wesleyel/bookify_rs/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/wesleyel/bookify_rs)](https://github.com/wesleyel/bookify_rs/releases)

Bookify-rs 是一个用 Rust 编写的 PDF 文档处理工具，主要用于生成适合双面打印的 PDF 文件。它提供了两种主要功能：小册子拼版和手动双面打印奇偶页面单独输出。

## 功能特点

### 1. 小册子拼版 (Booklet)

> [!TIP]  
> 打印时选择每页 2 或 4 页即可

- 将普通 PDF 转换为适合双面打印的小册子格式
- 支持每面 2 页或 4 页的排版布局
- 自动处理页面顺序和排列
- 保持原始 PDF 的页面质量

### 2. 手动双面打印 (Double-sided)

- 支持多种翻转类型：
  - `rr`: 奇偶页面都翻转
  - `nn`: 奇偶页面都不翻转
  - `rn`: 奇页面翻转，偶页面不翻转
  - `nr`: 偶页面翻转，奇页面不翻转
- 可选择输出奇数页或偶数页
- 自动生成临时文件（当未指定输出路径时）

## 安装方法

### 从源码安装

```bash
# 克隆仓库
git clone https://github.com/wesleyel/bookify-rs.git
cd bookify-rs

# 编译安装
cargo install --path .
```

### 使用 Cargo 安装

```bash
cargo install bookify-rs
```

## 使用说明

### 小册子拼版

基本用法：
```bash
bookify-rs booklet -i input.pdf -o output.pdf
```

完整参数：
```bash
bookify-rs booklet \
    -i input.pdf \                    # 输入 PDF 文件（必需）
    -o output.pdf \                   # 输出 PDF 文件（可选）
    --layout four-up                  # 排版布局类型（可选，默认 four-up）
```

### 手动双面打印

基本用法：
```bash
bookify-rs double-sided -i input.pdf -o output.pdf
```

完整参数：
```bash
bookify-rs double-sided \
    -i input.pdf \                    # 输入 PDF 文件（必需）
    -o output.pdf \                   # 输出 PDF 文件（可选）
    --flip-type rr \                  # 翻转类型（可选，默认 rr）
    --odd-even odd                    # 输出页面类型（可选，默认 odd）
```

## 参数说明

### 翻转类型 (--flip-type)
- `rr`: 奇偶页面都翻转
- `nn`: 奇偶页面都不翻转
- `rn`: 奇页面翻转，偶页面不翻转
- `nr`: 偶页面翻转，奇页面不翻转

### 输出页面类型 (--odd-even)
- `odd`: 输出奇数页
- `even`: 输出偶数页

## 注意事项

1. 输入文件必须是有效的 PDF 文件
2. 如果不指定输出文件路径，程序会自动生成临时文件
3. 建议在打印前预览输出文件，确保效果符合预期
4. 对于双面打印，请根据打印机的特性选择合适的翻转类型

## 技术实现

- 使用 `lopdf` 库处理 PDF 文件
- 使用 `clap` 库处理命令行参数
- 使用 `tempfile` 库管理临时文件
- 实现了完整的 PDF 页面操作和内容流处理

## 许可证

MIT

## 贡献指南

欢迎提交 Issue 和 Pull Request 来帮助改进这个项目。
