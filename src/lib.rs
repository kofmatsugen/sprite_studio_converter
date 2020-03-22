pub mod convert;
mod error;
mod sprite_sheet;

use amethyst_sprite_studio::traits::animation_file::AnimationFile;
use log::*;
use ron::ser::*;
use serde::Serialize;
use sprite_studio::{load_project, AnimationCells};
use std::{
    collections::BTreeMap,
    io::{BufWriter, Write},
    path::Path,
    str::FromStr,
};

pub fn convert_to_timeline<F, T>(
    dir_path: F,
    project_path: F,
) -> std::result::Result<(), failure::Error>
where
    F: AsRef<std::path::Path>,
    T: AnimationFile,
    T::PackKey: FromStr,
    T::AnimationKey: FromStr,
    <T::PackKey as FromStr>::Err: failure::Fail,
    <T::AnimationKey as FromStr>::Err: failure::Fail,
{
    let project_data = load_project(project_path.as_ref())?;
    convert_to_sprite_animation::<T>(&project_data, project_path.as_ref(), dir_path.as_ref())?;
    Ok(())
}

fn convert_to_sprite_animation<'a, T>(
    project_data: &'a sprite_studio::SpriteStudioData,
    project_path: &Path,
    output_dir: &Path,
) -> std::result::Result<(), failure::Error>
where
    T: AnimationFile,
    T::PackKey: FromStr,
    T::AnimationKey: FromStr,
    <T::PackKey as FromStr>::Err: failure::Fail,
    <T::AnimationKey as FromStr>::Err: failure::Fail,
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
    for (idx, cell_map) in project_data.cell_maps().enumerate() {
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

    let anim = convert::convert::<T>(project_data)?;
    data_to_file(anim, animation_dir.join("animation.anim.ron"))?;

    Ok(())
}

pub(crate) fn data_to_file<S, P>(data: S, path: P) -> std::result::Result<(), failure::Error>
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
