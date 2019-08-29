mod errors;
mod map;

use std::convert::TryInto;
use std::io::{BufWriter, Read, Write};
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
    let border_width = 4;
    let tile_size = 80;

    let current_tile_start_x = grid_start_x + tile_x * tile_size;
    let current_tile_start_y = grid_start_y + tile_y * tile_size;

    let tile = image.view(
        current_tile_start_x + border_width,
        current_tile_start_y + border_width,
        tile_size - border_width * 2,
        tile_size - border_width * 2,
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

fn get_tile_group(color: (u8, u8, u8)) -> u8 {
    let groups: [(u8, (u8, u8, u8)); 5] = [
        (1, (189, 187, 187)),
        (2, (236, 145, 187)),
        (3, (211, 171, 110)),
        (4, (99, 166, 184)),
        (0, (174, 131, 93)),
    ];

    let cmp_lambda = |cc1, cc2| {
        let c1 = u32::from(cc1);
        let c2 = u32::from(cc2);
        let delta_percentage = 15;
        c1 > (c2 - (c2 * delta_percentage) / 100) && c1 < (c2 + (c2 * delta_percentage) / 100)
    };

    for (group_type, group_color) in &groups {
        let c1 = cmp_lambda(color.0, group_color.0);
        let c2 = cmp_lambda(color.1, group_color.1);
        let c3 = cmp_lambda(color.2, group_color.2);

        if c1 && c2 && c3 {
            return *group_type;
        }
    }
    unreachable!("Tried to match tile of unknown avg color {:?}", color);
}

fn generate_monolith_map(image_data: &[u8]) -> Result<MonolithMap, MyError> {
    let image = image::load_from_memory(image_data)?;
    let mut map = MonolithMap::default();

    for x in 0..22usize {
        for y in 0..11usize {
            let avg_color =
                get_average_tile_color(&image, x.try_into().unwrap(), y.try_into().unwrap());
            let tile_group = get_tile_group(avg_color);
            map.set(x, y, tile_group);
        }
    }
    Ok(map)
}

fn get_monolith_map(image: &PathBuf) -> Result<MonolithMap, MyError> {
    let data_filepath = {
        let mut temp = image.clone();
        temp.set_extension("map");
        temp
    };
    if data_filepath.exists() {
        println!(
            "Using existing map data from {:?}.",
            data_filepath
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("???"))
        );
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
        println!(
            "Writing map data to {:?}.",
            data_filepath
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("???"))
        );
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
        println!(
            "Processing image {:?}...",
            image
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("???"))
        );
        if let Ok(data) = get_monolith_map(&image) {}
    }
}
