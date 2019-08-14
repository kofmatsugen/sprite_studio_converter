mod convert;

use crate::convert::*;
use amethyst::animation::AnimationHierarchyPrefab;
use amethyst::{assets::Prefab, core::Transform};
use amethyst_sprite_studio::SpriteAnimation;
use log::*;
use ron::ser::*;
use serde::{de::DeserializeOwned, Serialize};
use sprite_studio::*;
use std::io::{BufReader, BufWriter, Write};

pub fn convert_to_file<P>(
    dir_path: P,
    project_path: P,
) -> std::result::Result<(), Box<std::error::Error>>
where
    P: AsRef<std::path::Path>,
{
    std::fs::create_dir_all(dir_path.as_ref())?;

    let project_data = load_project(project_path)?;

    // transform animation
    let transform_dir = dir_path.as_ref().join("transform");
    std::fs::create_dir_all(&transform_dir)?;
    for (pack, animations) in make_transform_animations(&project_data) {
        let transform_dir = transform_dir.join(pack);
        std::fs::create_dir_all(&transform_dir)?;
        for (name, animation) in animations {
            let string = data_to_string(animation)?;
            let converted_path = transform_dir.join(name + ".ron");
            let file = std::fs::File::create(converted_path)?;
            let mut buff = BufWriter::new(file);
            buff.write(string.as_bytes())?;
        }
    }

    // sprite sheets
    let sprite_sheet_dir = dir_path.as_ref().join("sprite_sheet");
    std::fs::create_dir_all(&sprite_sheet_dir)?;
    for (name, sheet) in make_sprite_sheets(&project_data) {
        let string = data_to_string(sheet)?;
        let converted_path = sprite_sheet_dir.join(name + ".ron");
        let file = std::fs::File::create(converted_path)?;
        let mut buff = BufWriter::new(file);
        buff.write(string.as_bytes())?;
    }

    // sprite renders
    let sprite_render_animation_dir = dir_path.as_ref().join("sprite_render_animation");
    std::fs::create_dir_all(&sprite_render_animation_dir)?;
    for (pack, renders) in make_sprite_renders(&project_data) {
        let transform_dir = sprite_render_animation_dir.join(pack);
        std::fs::create_dir_all(&transform_dir)?;
        for (name, render) in renders {
            let string = data_to_string(render)?;
            let converted_path = transform_dir.join(name + ".ron");
            let file = std::fs::File::create(converted_path)?;
            let mut buff = BufWriter::new(file);
            buff.write(string.as_bytes())?;
        }
    }

    let sprite_render_dir = dir_path.as_ref().join("sprite_render");
    std::fs::create_dir_all(&sprite_render_dir)?;
    for (pack, renders) in make_sprite_render(&project_data) {
        let sprite_render_dir = sprite_render_dir.join(pack);
        std::fs::create_dir_all(&sprite_render_dir)?;
        for (name, render) in renders {
            let string = data_to_string(render)?;
            let converted_path = sprite_render_dir.join(name + ".ron");
            let file = std::fs::File::create(converted_path)?;
            let mut buff = BufWriter::new(file);
            buff.write(string.as_bytes())?;
        }
    }

    // animation hierarchy
    let sprite_sheet_dir = dir_path.as_ref().join("hierarchy");
    std::fs::create_dir_all(&sprite_sheet_dir)?;
    for (name, hierarchy) in make_animation_hierarchy(&project_data) {
        let string = data_to_string(hierarchy)?;
        let converted_path = sprite_sheet_dir.join(name + ".ron");
        let file = std::fs::File::create(converted_path)?;
        let mut buff = BufWriter::new(file);
        buff.write(string.as_bytes())?;
    }

    if let Some(pack) = project_data.packs().next() {
        let mut prefab = Prefab::<SpriteAnimation>::new();

        for part in pack.parts() {
            let pack_name: String = pack.name().into();
            let part_name: String = part.name().into();

            let sheet_path = dir_path
                .as_ref()
                .join("sprite_sheet")
                .join(pack_name + ".ron");
            let sheet = file_to_data(sheet_path);

            let cell_path = dir_path
                .as_ref()
                .join("sprite_render")
                .join(pack.name())
                .join(part_name + ".ron");
            let cell = file_to_data(cell_path);

            let mut animation = match (sheet, cell) {
                (Ok(sheet), Ok(cell)) => {
                    SpriteAnimation::new(Some(sheet), Some(cell), Transform::default())
                }
                (_, _) => SpriteAnimation::new(None, None, Transform::default()),
            };
            if part.index() == 0 {
                let pack_name: String = pack.name().into();

                let hierarchy_path = dir_path.as_ref().join("hierarchy").join(pack_name + ".ron");
                let transform_hierarchy: AnimationHierarchyPrefab<Transform> =
                    file_to_data(&hierarchy_path)?;
                animation.set_transform_hierarchy(transform_hierarchy);
                info!("main part: {}", part.index());
                prefab.main(animation.into());
            } else if part.parent() < 0 {
                info!("non parent part: {}", part.index());
                prefab.add(None, animation.into());
            } else {
                info!("parent[{}] part: {}", part.parent(), part.index());
                prefab.add((part.parent() as usize).into(), animation.into());
            }
        }
        let converted_path = dir_path.as_ref().join("project");
        std::fs::create_dir_all(&converted_path)?;
        let converted_path = converted_path.join("test.ron");
        let file = std::fs::File::create(converted_path)?;
        let string = data_to_string(prefab)?;
        let mut buff = BufWriter::new(file);
        buff.write(string.as_bytes())?;
    }

    Ok(())
}

fn data_to_string<S: Serialize>(data: S) -> Result<String> {
    let config = PrettyConfig {
        depth_limit: std::usize::MAX,
        new_line: "\n".into(),
        indentor: "\t".into(),
        separate_tuple_members: true,
        enumerate_arrays: true,
    };
    let string = ron::ser::to_string_pretty(&data, config)?;
    Ok(string)
}

fn file_to_data<D: DeserializeOwned, P>(path: P) -> std::result::Result<D, Box<std::error::Error>>
where
    P: AsRef<std::path::Path>,
{
    let path = path.as_ref();
    let file = std::fs::File::open(path)?;
    let buf_reader = BufReader::new(file);
    let data = ron::de::from_reader(buf_reader)?;

    Ok(data)
}
