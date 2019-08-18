mod key_frame;

use amethyst::animation::{
    AnimationHierarchyPrefab, AnimationPrefab, SpriteRenderChannel, TransformChannel,
};
use amethyst::core::Transform;
use amethyst::renderer::{
    formats::texture::TexturePrefab,
    sprite::{
        prefab::{SpriteRenderPrefab, SpriteSheetPrefab},
        SpriteList, SpritePosition, SpriteRender, Sprites,
    },
    ImageFormat,
};
use key_frame::*;
use sprite_studio::*;
use std::collections::BTreeMap;

pub(crate) fn make_sprite_sheets(data: &SpriteStudioData) -> BTreeMap<String, SpriteSheetPrefab> {
    let mut sprite_sheets = BTreeMap::new();
    for cell_map in data.cell_maps() {
        let sprites = make_sprite_sheet(cell_map);
        sprite_sheets.insert(cell_map.name().into(), sprites);
    }
    sprite_sheets
}

pub(crate) fn make_sprite_renders(
    data: &SpriteStudioData,
) -> BTreeMap<String, BTreeMap<String, AnimationPrefab<SpriteRender>>> {
    let mut packs = BTreeMap::new();
    for pack in data.packs() {
        let mut animations = BTreeMap::new();
        for anim in pack.animations() {
            let mut animation = AnimationPrefab::default();
            for pa in anim.part_animes() {
                let timeline = TimeLine::<i32>::new(anim.setting().fps(), pa);
                let part = pack
                    .get_part_by_name(pa.name())
                    .map(|part| part.index() as usize)
                    .expect(&format!("not exist {}", pa.name()));
                animation.samplers.push((
                    part,
                    SpriteRenderChannel::SpriteIndex,
                    timeline.sprite_renders(data),
                ));
            }
            animations.insert(anim.name().into(), animation);
        }
        packs.insert(pack.name().into(), animations);
    }
    packs
}

pub(crate) fn make_sprite_render(
    data: &SpriteStudioData,
) -> BTreeMap<String, BTreeMap<String, SpriteRenderPrefab>> {
    let mut packs = BTreeMap::new();
    for pack in data.packs() {
        let mut animations = BTreeMap::new();
        for anim in pack.animations() {
            for pa in anim.part_animes() {
                let render = TimeLine::<i32>::new(anim.setting().fps(), pa).sprite_render(data);
                if let Some(render) = render {
                    animations.insert(pa.name().into(), render);
                }
            }
        }
        packs.insert(pack.name().into(), animations);
    }
    packs
}

pub(crate) fn make_animation_hierarchy(
    data: &SpriteStudioData,
) -> BTreeMap<String, AnimationHierarchyPrefab<Transform>> {
    let mut hierarchies = BTreeMap::new();
    for pack in data.packs() {
        let mut hierarchy = AnimationHierarchyPrefab::default();
        for part in pack.parts() {
            let part_index = part.index() as usize;
            hierarchy.nodes.push((part_index, part_index));
        }
        hierarchies.insert(pack.name().into(), hierarchy);
    }
    hierarchies
}

pub(crate) fn make_transform_animations(
    data: &SpriteStudioData,
) -> BTreeMap<String, BTreeMap<String, AnimationPrefab<Transform>>> {
    let mut packs = BTreeMap::new();
    for pack in data.packs() {
        let mut animations = BTreeMap::new();
        for anim in pack.animations() {
            let mut animation = AnimationPrefab::default();
            for pa in anim.part_animes() {
                let timeline = TimeLine::<i32>::new(anim.setting().fps(), pa);
                let part = pack
                    .get_part_by_name(pa.name())
                    .map(|part| part.index() as usize)
                    .expect(&format!("not exist {}", pa.name()));
                let position = timeline.positions();
                if position.output.len() > 0 {
                    animation
                        .samplers
                        .push((part, TransformChannel::Translation, position));
                }
                let rotations = timeline.rotations();
                if rotations.output.len() > 0 {
                    animation
                        .samplers
                        .push((part, TransformChannel::Rotation, rotations));
                }
            }
            animations.insert(anim.name().into(), animation);
        }
        packs.insert(pack.name().into(), animations);
    }
    packs
}

fn make_sprite_sheet(cell_map: &AnimationCells) -> SpriteSheetPrefab {
    let mut sprites = vec![];
    for cell in cell_map.cells() {
        let (x, y) = cell.position();
        let (x, y) = (x as u32, y as u32);
        let (width, height) = cell.size();
        let (width, height) = (width as u32, height as u32);
        let (pivot_x, pivot_y) = cell.pivot();
        let flip_horizontal = false;
        let flip_vertical = false;
        let sprite = SpritePosition {
            x,
            y,
            width,
            height,
            flip_horizontal,
            flip_vertical,
            offsets: Some([pivot_x, pivot_y]),
        };
        sprites.push(sprite);
    }

    let (texture_width, texture_height) = cell_map.pixel_size();
    let (texture_width, texture_height) = (texture_width as u32, texture_height as u32);
    let sprites = vec![Sprites::List(SpriteList {
        texture_width,
        texture_height,
        sprites,
    })];

    let texture_prefab = TexturePrefab::File(
        cell_map.image_path().into(),
        Box::new(ImageFormat::default()),
    );

    SpriteSheetPrefab::Sheet {
        texture: texture_prefab,
        sprites: sprites,
        name: Some(cell_map.name().into()),
    }
}
