#![allow(warnings)]

use downcast_rs::__std::ffi::{OsStr, OsString};
use log::{debug, error, info};
use spacegame::assets::sprite::{
    load_texels, Packed, PackedSpriteAsset, SamplerDef, SpriteAssetMetadata,
};
use std::collections::HashMap;
use std::fs::FileType;
use std::path::PathBuf;

fn load_metadata(base_path: PathBuf, asset_name: &str) -> SpriteAssetMetadata {
    let metadata_path = base_path.join(asset_name).with_extension("json");
    info!(
        "Will load {:?} metadata at {}",
        asset_name,
        metadata_path.display()
    );
    let metadata_str = std::fs::read_to_string(metadata_path);

    match metadata_str {
        Ok(metadata_str) => serde_json::from_str::<SpriteAssetMetadata>(&metadata_str)
            .unwrap_or_else(|e| {
                error!(
                    "Cannot deserialize Metadata file, will use default instead = {:?}",
                    e
                );
                SpriteAssetMetadata::default()
            }),
        Err(_) => {
            info!(
                "No metadata file for {}, Will use default instead.",
                asset_name
            );
            SpriteAssetMetadata::default()
        }
    }
}

fn get_packed_sprite(asset_name: &str) -> PackedSpriteAsset {
    let base_path = PathBuf::from("assets/sprites");
    let asset_path = base_path.join(asset_name);
    let metadata = load_metadata(base_path.clone(), asset_name);
    let (w, h, data) = load_texels(asset_path).unwrap();

    PackedSpriteAsset {
        w,
        h,
        data,
        sampler: metadata.sampler,
    }
}

fn main() {
    let dirs = vec![
        // "./assets/sprites",
        // "./assets/sprites/background2",
        // "./assets/sprites/background3",
        // "./assets/sprites/explosion4",
        // "./assets/sprites/explosion4",
        // "./assets/sprites/windshield_wiper",
        "./assets/sprites/spaceships",
        "./assets/sprites/spaceships/Projectiles",
    ];

    let to_pack = dirs
        .iter()
        .flat_map(|d| {
            let paths = std::fs::read_dir(d).unwrap();
            paths
                // only files.
                .filter(|p| {
                    let p = p.as_ref().unwrap();
                    match p.file_type() {
                        Ok(t) => t.is_file(),
                        _ => false,
                    }
                })
                // only PNG
                .filter(|p| {
                    let p = p.as_ref().unwrap();
                    match p.path().extension() {
                        Some(ext) => ext.to_os_string() == OsString::from("png"),
                        _ => false,
                    }
                })
                .map(|p| {
                    p.unwrap().path().display().to_string()["./assets/sprites/".len()..].to_string()
                })
        })
        .collect::<Vec<_>>();

    let content = {
        let mut m = HashMap::new();

        for n in &to_pack {
            m.insert(n.clone(), get_packed_sprite(n.as_str()));
        }
        //  let n = "spaceships/blue_05.png";
        m
    };
    let packed = Packed { content };

    let res = bincode::serialize(&packed).unwrap();
    std::fs::write("packed.bin", res).unwrap();
}
