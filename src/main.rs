#[macro_use]
extern crate log;

mod atlas;
mod images;

use crate::images::ImageDefinition as ImageDefinitionExt;
use image::ImageResult;
use serde::{Deserialize, Serialize};
use simple_logger;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

use images::{Config, build};

#[derive(Deserialize, Debug)]
struct TextureList {
    images: Vec<ImageDefinition>,
    width: u32,
    height: u32,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ImageDefinition {
    Path(PathBuf),
    Ext(ImageDefinitionExt),
}

impl Into<ImageDefinitionExt> for ImageDefinition {
    fn into(self) -> ImageDefinitionExt {
        use ImageDefinition::*;
        match self {
            Ext(e) => e,
            Path(p) => ImageDefinitionExt::new(p),
        }
    }
}

#[derive(StructOpt, Debug)]
struct Ops {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    out_texture: PathBuf,
    #[structopt(parse(from_os_str))]
    out_map: PathBuf,

    #[structopt(short="d", long="--dir", parse(from_os_str), default_value="")]
    base_dir: PathBuf,
}

fn main() -> ImageResult<()> {
    simple_logger::init().unwrap();
    let ops = Ops::from_args();
    let config_file = File::open(&ops.input)?;
    let textures: TextureList = serde_json::from_reader(config_file)
        .map_err(|e| -> std::io::Error { e.into() })?;
    let mut texts = Vec::with_capacity(textures.images.len());
    for i in textures.images {
        let texture: ImageDefinitionExt = i.into();
        info!("{:?}", texture);
        texts.push(texture);
    }

    let config = Config{
        width: textures.width,
        height: textures.height,
        input: texts,
        base_dir: ops.base_dir,
        output_image: ops.out_texture,
        output_map: ops.out_map,
        border: 1,
    };

    build(&config)?;
    Ok(())
}
