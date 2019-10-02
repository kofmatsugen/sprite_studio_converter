pub mod sprite_sheet;
pub mod timeline;
mod types;

use amethyst_sprite_studio::SpriteAnimation;
use log::*;
use ron::ser::*;
use serde::{Deserialize, Serialize};
use sprite_studio::{load_project, AnimationCells};
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn convert_to_timeline<'de, P, U>(
    dir_path: P,
    project_path: P,
) -> std::result::Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<std::path::Path>,
    U: amethyst_sprite_studio::traits::FromUser + Serialize + Deserialize<'de>,
{
    let project_data = load_project(project_path.as_ref())?;
    convert_to_sprite_animation::<U>(&project_data, project_path.as_ref(), dir_path.as_ref())?;
    Ok(())
}

fn convert_to_sprite_animation<'de, U>(
    data: &sprite_studio::SpriteStudioData,
    project_path: &Path,
    output_dir: &Path,
) -> std::result::Result<(), Box<dyn std::error::Error>>
where
    U: amethyst_sprite_studio::traits::FromUser + Serialize + Deserialize<'de>,
{
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

    let mut cell_name_dict = vec![];
    for (idx, cell_map) in data.cell_maps().enumerate() {
        // スプライトの分割情報を生成
        // ファイル生成時はIDのファイル名で生成する
        info!("{}: {} {:?}", idx, cell_map.name(), cell_map.image_path());

        // 画像を生成パス内にコピー
        let from = project_dir.join(cell_map.image_path());
        let img_path = image_dir.join(format!("sprite{:03}.png", idx));

        info!("{:?} => {:?}", from, img_path);

        std::fs::copy(from, img_path)?;

        cell_name_dict.push(make_cell_name_dict(cell_map));
        let sheet = sprite_sheet::make_sprite_sheet(cell_map);
        let sheet_path = sheet_dir.join(format!("sprite{:03}.sheet.ron", idx));
        data_to_file(sheet, sheet_path)?;
    }

    let mut pack_id_map = BTreeMap::new();
    for (pack_idx, pack) in data.packs().enumerate() {
        let mut anim_id_map = BTreeMap::new();

        for (anim_idx, anim) in pack.animations().enumerate() {
            anim_id_map.insert(anim.name(), anim_idx);
        }
        pack_id_map.insert(pack.name(), (pack_idx, anim_id_map));
    }

    // パック名とアニメーション名をIDに変換するためのBTreeMap
    for (pack_idx, pack) in data.packs().enumerate() {
        info!("{}", pack.name());
        let anim_pack_dir = animation_dir.join(format!("pack{:03}", pack_idx));
        std::fs::create_dir_all(&anim_pack_dir)?;
        let mut pack_index = BTreeMap::new();
        // あとでパーツをIDで管理するために名前と結びつくようにしておく
        for part in pack.parts() {
            pack_index.insert(part.name(), part);
        }
        for (idx, anim) in pack.animations().enumerate() {
            info!("\t{}", anim.name());
            let count = anim.setting().count() as usize;
            let fps = anim.setting().fps();
            let mut animations = SpriteAnimation::<U>::new(fps, count);
            for pa in anim.part_animes() {
                let part = pack_index[pa.name()];
                let id = part.index() as usize;
                let parent = if part.parent() < 0 {
                    None
                } else {
                    Some(part.parent() as usize)
                };

                let refference_animation = part.refference_animation().and_then(|(pack, anim)| {
                    pack_id_map.get(pack).and_then(|(pack_id, anim_map)| {
                        anim_map.get(anim).map(|anim_id| (*pack_id, *anim_id))
                    })
                });

                let ref_pack_id = refference_animation.map(|(pack_id, _)| pack_id);
                let ref_anim_id = refference_animation.map(|(_, anim_id)| anim_id);

                let tl = timeline::part_anime_to_timeline(count, pa, &cell_name_dict)
                    .part_id(id)
                    .parent_id(parent)
                    .part_type(types::convert_part_type(part.part_type()))
                    .bounds(types::convert_bounds(part.bounds()))
                    .ref_pack_id(ref_pack_id)
                    .ref_anim_id(ref_anim_id)
                    .build();
                animations.add_timeline(tl);
            }
            let animation_path = anim_pack_dir.join(format!("animation{:03}.anim.ron", idx));
            data_to_file(animations, animation_path)?;
        }
    }

    Ok(())
}

pub(crate) fn data_to_file<S, P>(
    data: S,
    path: P,
) -> std::result::Result<(), Box<dyn std::error::Error>>
where
    S: Serialize,
    P: AsRef<std::path::Path> + std::fmt::Debug,
{
    let config = PrettyConfig {
        depth_limit: std::usize::MAX,
        new_line: "\n".into(),
        indentor: "\t".into(),
        separate_tuple_members: false,
        enumerate_arrays: true,
    };
    info!("save: {:?}", path);
    let string = ron::ser::to_string_pretty(&data, config)?;
    let file = std::fs::File::create(path)?;
    let mut buff = BufWriter::new(file);
    buff.write(string.as_bytes())?;
    Ok(())
}

fn make_cell_name_dict(cell_map: &AnimationCells) -> BTreeMap<String, usize> {
    let mut cell_name_dict = BTreeMap::new();

    for (idx, cell) in cell_map.cells().enumerate() {
        cell_name_dict.insert(cell.name().into(), idx);
    }

    cell_name_dict
}
