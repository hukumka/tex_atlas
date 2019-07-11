use crate::atlas::{Size, Rect, AtlasBuilder};
use std::fs::File;
use image;
use image::{GenericImageView, GenericImage, RgbaImage, DynamicImage, Pixel};
use image::ImageResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Default)]
pub struct ImageDefinition {
    path: PathBuf,
    #[serde(default)]
    repeat: bool,
}

impl ImageDefinition {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }
}

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub input: Vec<ImageDefinition>,
    pub base_dir: PathBuf,
    pub output_image: PathBuf,
    pub output_map: PathBuf,
    pub border: u32,
}

pub fn build(config: &Config) -> ImageResult<()> {
    let mut images = HashMap::with_capacity(config.input.len());
    let mut rects = Vec::with_capacity(config.input.len());
    for def in &config.input {
        let image = image::open(config.base_dir.join(&def.path))
            .map_err(|e|{
                error!("Failed to process image {:?}: {}", &def.path, &e);
                e
            })?;
        let size = Size {
            width: image.width() + config.border,
            height: image.height() + config.border,
        };
        let name = def.path.clone().into_os_string().to_string_lossy().to_string();
        images.insert(name.clone(), image);
        rects.push((name, size));
    }
    let mut builder = AtlasBuilder::new(config.width, config.height);
    builder.build(rects).expect("Could not fit images into atlas of specified size");
    let bound_size = builder.min_bounding_rect();
    let map = builder.get_map();

    let mut buffer = RgbaImage::new(bound_size.width, bound_size.height);
    for (name, image) in &images{
        let mut rect = *map.textures.get(name).unwrap_or_else(|| panic!("Image {:?} has no associated space!", name));
        rect.size.width -= config.border;
        rect.size.height -= config.border;
        copy_to_rgba(image, &mut buffer, rect);
    }
    buffer.save(config.output_image.clone())?;

    let mut out = File::create(&config.output_map)?;
    serde_json::to_writer_pretty(&mut out, &map).unwrap();
    Ok(())
}

fn copy_to_rgba(from: &DynamicImage, into: &mut RgbaImage, rect: Rect){
    assert!(rect.left + rect.size.width <= into.width());
    assert!(rect.top + rect.size.height <= into.height());
    assert_eq!(rect.size.width, from.width());
    assert_eq!(rect.size.height, from.height());

    for y in 0..rect.size.height{
        for x in 0..rect.size.width{
            let pixel = from.get_pixel(x, y);
            into.put_pixel(rect.left + x, rect.top + y, pixel);
        }
    }
}
