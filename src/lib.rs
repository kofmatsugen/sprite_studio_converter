pub mod sprite_sheet;
pub mod timeline;

use log::*;
use ron::ser::*;
use serde::Serialize;
use sprite_studio::load_project;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn convert_to_timeline<P>(
    dir_path: P,
    project_path: P,
) -> std::result::Result<(), Box<std::error::Error>>
where
    P: AsRef<std::path::Path>,
{
    let project_data = load_project(project_path.as_ref())?;
    convert_to_sprite_animation(&project_data, project_path.as_ref(), dir_path.as_ref())?;
    Ok(())
}

fn convert_to_sprite_animation(
    data: &sprite_studio::SpriteStudioData,
    project_path: &Path,
    output_dir: &Path,
) -> std::result::Result<(), Box<std::error::Error>> {
    let project_name = project_path.file_stem().unwrap();
    let project_dir = project_path.parent().unwrap();

    let output_project_dir = output_dir.join(project_name);
    let image_dir = output_project_dir.join("image");
    let sheet_dir = output_project_dir.join("sheet");
    let animation_dir = output_project_dir.join("animation");
    info!("{:?}", output_project_dir);
    std::fs::create_dir_all(&output_project_dir)?;
    std::fs::create_dir_all(&image_dir)?;
    std::fs::create_dir_all(&sheet_dir)?;
    std::fs::create_dir_all(&animation_dir)?;

    for (idx, cell_map) in data.cell_maps().enumerate() {
        // スプライトの分割情報を生成
        // ファイル生成時はIDのファイル名で生成する
        info!("{}: {} {:?}", idx, cell_map.name(), cell_map.image_path());

        // 画像を生成パス内にコピー
        let from = project_dir.join(cell_map.image_path());
        let img_path = image_dir.join(format!("sprite{:03}.png", idx));

        info!("{:?} => {:?}", from, img_path);

        std::fs::copy(from, img_path)?;

        let sheet = sprite_sheet::make_sprite_sheet(cell_map);
        let sheet_path = sheet_dir.join(format!("sprite{:03}.sheet.ron", idx));
        data_to_file(sheet, sheet_path)?;
    }

    for pack in data.packs() {
        info!("{}", pack.name());
        let mut pack_index = std::collections::BTreeMap::new();
        // あとでパーツをIDで管理するために名前と結びつくようにしておく
        for part in pack.parts() {
            pack_index.insert(part.name(), part);
        }
        let mut animations = vec![];
        for anim in pack.animations() {
            info!("\t{}", anim.name());
            let count = anim.setting().count() as usize;
            for pa in anim.part_animes() {
                let id = pack_index[pa.name()].index() as usize;
                let parent = if pack_index[pa.name()].parent() < 0 {
                    None
                } else {
                    Some(pack_index[pa.name()].parent() as usize)
                };
                let tl = timeline::part_anime_to_timeline::<(), _>(count, pa, id, parent);
                animations.push(tl);
            }
        }

        let animation_path = animation_dir.join(format!("{}.anim.ron", pack.name()));
        data_to_file(animations, animation_path)?;
    }

    Ok(())
}

pub(crate) fn data_to_file<S, P>(
    data: S,
    path: P,
) -> std::result::Result<(), Box<std::error::Error>>
where
    S: Serialize,
    P: AsRef<std::path::Path> + std::fmt::Debug,
{
    let config = PrettyConfig {
        depth_limit: std::usize::MAX,
        new_line: "\n".into(),
        indentor: "\t".into(),
        separate_tuple_members: true,
        enumerate_arrays: true,
    };
    info!("save: {:?}", path);
    let string = ron::ser::to_string_pretty(&data, config)?;
    let file = std::fs::File::create(path)?;
    let mut buff = BufWriter::new(file);
    buff.write(string.as_bytes())?;
    Ok(())
}
