use std::path::PathBuf;

use crate::calc::LayoutType;
use clap::{Parser, Subcommand, ValueEnum};

/// 翻页方向
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ReadingDirection {
    /// 从左到右翻页
    #[value(name = "left-to-right")]
    LeftToRight,
    /// 从右到左翻页  
    #[value(name = "right-to-left")]
    RightToLeft,
}

/// 翻转方向
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum FlipDirection {
    /// 短边翻转
    #[value(name = "short-edge")]
    ShortEdge,
    /// 长边翻转
    #[value(name = "long-edge")]
    LongEdge,
}

/// 输出奇数或偶数页
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OddEven {
    /// 奇数页
    #[value(name = "odd")]
    Odd,
    /// 偶数页
    #[value(name = "even")]
    Even,
}

/// 小册子拼版选项
#[derive(Debug, Parser)]
pub struct BookletOptions {
    /// 输入 PDF 文件
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub input: PathBuf,

    /// 输出 PDF 文件
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// 排版布局类型
    #[arg(long, value_enum, default_value = "four-up")]
    pub layout: LayoutType,
}

/// 双面打印选项
#[derive(Debug, Parser)]
pub struct DoubleSidedOptions {
    /// 输入 PDF 文件
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub input: PathBuf,

    /// 输出 PDF 文件
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// 翻页方向
    #[arg(long, value_enum, default_value = "left-to-right")]
    pub reading_direction: ReadingDirection,

    /// 翻转方向
    #[arg(long, value_enum, default_value = "long-edge")]
    pub flip_direction: FlipDirection,

    /// 输出奇数或偶数页
    #[arg(long, value_enum, default_value = "odd")]
    pub odd_even: OddEven,
}

/// 命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// 子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 小册子拼版：将 PDF 转换为适合双面打印的小册子格式
    #[command(name = "booklet")]
    Booklet(BookletOptions),

    /// 双面打印：将 PDF 转换为适合双面打印的格式
    #[command(name = "double-sided")]
    DoubleSided(DoubleSidedOptions),
}
