mod errors;
mod map;

use std::io::Read;
use std::path::PathBuf;

use image::GenericImageView;

use errors::MyError;
use map::MonolithMap;

fn get_image_files() -> Vec<PathBuf> {
    let cwd = std::env::current_dir().expect("Failed to obtain CWD.");
    let contents = cwd.read_dir().expect("Failed to read from CWD.");
    let mut results = vec![];
    for entry in contents {
        let path = entry.expect("Failed to get DirEntry information").path();
        if let Some(extension) = path.extension() {
            if extension
                .to_str()
                .expect("Failed to get DirEntry extension.")
                .to_lowercase()
                == "png"
            {
                results.push(path);
            }
        }
    }
    results
}

fn get_average_tile_color(image: &image::DynamicImage, tile_x: u32, tile_y: u32) -> (u8, u8, u8) {
    let grid_start_x = 80;
    let grid_start_y = 80;
    let border_width = 3;
    let tile_size = 80;

    let current_tile_start_x = grid_start_x + border_width + (tile_x * tile_size);
    let current_tile_start_y = grid_start_y + border_width + (tile_y * tile_size);

    let tile = image.view(
        current_tile_start_x,
        current_tile_start_y,
        tile_size,
        tile_size,
    );
    let mut r: u32 = 0;
    let mut g: u32 = 0;
    let mut b: u32 = 0;
    let mut count: u32 = 0;
    for pixel in tile.pixels() {
        count += 1;

        let pr = u32::from(pixel.2[0]);
        let pg = u32::from(pixel.2[1]);
        let pb = u32::from(pixel.2[2]);

        r += pr * pr;
        g += pg * pg;
        b += pb * pb;
    }

    (
        f64::from(r / count).sqrt() as u8,
        f64::from(g / count).sqrt() as u8,
        f64::from(b / count).sqrt() as u8,
    )
}

fn generate_monolith_map(image_data: &[u8]) -> Result<MonolithMap, MyError> {
    let image = image::load_from_memory(image_data)?;
    println!("Got image of size {:?}", image.dimensions());
    Ok(MonolithMap::default())
}

fn get_monolith_data(image: &PathBuf) -> Result<MonolithMap, MyError> {
    let data_filepath = {
        let mut temp = image.clone();
        temp.set_extension("map");
        temp
    };
    if data_filepath.exists() {
        let mut data_file = std::fs::File::open(&data_filepath)?;
        let mut buffer = String::new();
        data_file.read_to_string(&mut buffer)?;
        Ok(serde_json::from_str::<MonolithMap>(&buffer)?)
    } else {
        let image_data = {
            let mut temp = std::fs::File::open(&image)?;
            let mut buffer = Vec::new();
            temp.read_to_end(&mut buffer)?;
            buffer
        };

        let map_data = generate_monolith_map(&image_data)?;

        let data_file = std::fs::File::create(&data_filepath)?;
        serde_json::to_writer(data_file, &map_data)?;

        Ok(map_data)
    }
}

fn main() {
    let found_image_files = get_image_files();
    if found_image_files.is_empty() {
        println!("No PNG images found.");
        return;
    }

    for image in found_image_files {
        if let Ok(data) = get_monolith_data(&image) {}
    }
}
