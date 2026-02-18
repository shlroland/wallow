mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Deserialize, Debug)]
struct WallpaperData {
    path: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    data: Vec<WallpaperData>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Hello { name } => {
            println!("Hello, {}!", name);
        }
        Commands::Random => {
            fetch_random_wallpaper()?;
        }
    }

    Ok(())
}

fn fetch_random_wallpaper() -> Result<(), Box<dyn Error>> {
    let url = "https://wallhaven.cc/api/v1/search?resolutions=3840x2160&sorting=random";

    println!("正在搜索 4K 壁纸...");
    let response: ApiResponse = reqwest::blocking::get(url)?.json()?;

    if let Some(wallpaper) = response.data.first() {
        println!("发现壁纸: {}", wallpaper.path);

        println!("正在下载...");
        let mut img_response = reqwest::blocking::get(&wallpaper.path)?;
        let mut file = File::create("wallpaper.jpg")?;

        img_response.copy_to(&mut file)?;

        println!("下载完成！已保存为 wallpaper.jpg");
    } else {
        println!("未找到符合条件的壁纸。");
    }

    Ok(())
}
