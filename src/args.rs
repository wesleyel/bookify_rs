use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// 翻页方向
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ReadingDirection {
    /// 从左到右翻页
    LeftToRight,
    /// 从右到左翻页  
    RightToLeft,
}

/// 翻转方向
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum FlipDirection {
    /// 短边翻转
    ShortEdge,
    /// 长边翻转
    LongEdge,
}

/// 拼版方法
#[derive(Clone, Debug, Parser, ValueEnum)]
pub enum Method {
    /// 小册子拼版
    Booklet,
    /// 双面打印拼版
    DoubleSided,
}

/// 命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 输入 PDF 文件
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub input: PathBuf,

    /// 输出 PDF 文件
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// 目标总页数（默认16页）
    #[arg(long, default_value = "16")]
    pub pages: usize,

    /// 翻页方向
    #[arg(long, value_enum, default_value = "left-to-right")]
    pub reading_direction: ReadingDirection,

    /// 翻转方向
    #[arg(long, value_enum, default_value = "short-edge")]
    pub flip_direction: FlipDirection,
}
