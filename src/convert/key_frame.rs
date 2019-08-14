// Attribute 変換
mod cell;
mod position;
mod rotation;
mod scale;
mod user;

use cell::*;
use position::*;
use rotation::*;
use scale::*;
use sprite_studio::*;
use user::*;

use amethyst::animation::{
    InterpolationFunction, Sampler, SamplerPrimitive, SpriteRenderPrimitive,
};
use amethyst::renderer::sprite::prefab::{SpriteRenderPrefab, SpriteSheetReference};

#[derive(Debug)]
pub(crate) enum KeyFrame<U = i32> {
    Cell {
        map_id: u32,
        part_name: String,
    },
    Position {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
    },
    Rotation {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
    },
    Scale {
        x: Option<f32>,
        y: Option<f32>,
    },
    User {
        paramater: U,
    },
}

#[derive(Debug)]
pub(crate) struct Key<U = i32> {
    time: f32,
    key_frame: KeyFrame<U>,
    interpolation: Interpolation,
}

#[derive(Debug)]
pub(crate) struct TimeLine<U = i32> {
    key_frames: Vec<Key<U>>,
}

impl<U> TimeLine<U>
where
    U: From<i32> + Default,
{
    pub(crate) fn new(fps: u32, part_anime: &PartAnime) -> Self {
        let mut timeline = TimeLine { key_frames: vec![] };

        for attr in part_anime.attributes() {
            match attr.tag() {
                AttributeTag::Cell => {
                    let mut cells = to_keys(fps, attr.keys(), build_cell);
                    timeline.key_frames.append(&mut cells);
                }
                AttributeTag::User => {
                    let mut users = to_keys(fps, attr.keys(), build_users);
                    timeline.key_frames.append(&mut users);
                }
                AttributeTag::Posx => {
                    let mut posx = to_keys(fps, attr.keys(), build_position::<PosX>);
                    timeline.key_frames.append(&mut posx);
                }
                AttributeTag::Posy => {
                    let mut posy = to_keys(fps, attr.keys(), build_position::<PosY>);
                    timeline.key_frames.append(&mut posy);
                }
                AttributeTag::Posz => {
                    let mut posz = to_keys(fps, attr.keys(), build_position::<PosZ>);
                    timeline.key_frames.append(&mut posz);
                }
                AttributeTag::Prio => {
                    let mut posz = to_keys(fps, attr.keys(), build_position::<PosZ>);
                    timeline.key_frames.append(&mut posz);
                }
                AttributeTag::Sclx => {
                    let mut scale_x = to_keys(fps, attr.keys(), build_scale::<ScaleX>);
                    timeline.key_frames.append(&mut scale_x);
                }
                AttributeTag::Scly => {
                    let mut scale_y = to_keys(fps, attr.keys(), build_scale::<ScaleY>);
                    timeline.key_frames.append(&mut scale_y);
                }
                AttributeTag::Rotx => {
                    let mut rot_x = to_keys(fps, attr.keys(), build_rotation::<RotX>);
                    timeline.key_frames.append(&mut rot_x);
                }
                AttributeTag::Roty => {
                    let mut rot_y = to_keys(fps, attr.keys(), build_rotation::<RotY>);
                    timeline.key_frames.append(&mut rot_y);
                }
                AttributeTag::Rotz => {
                    let mut rot_z = to_keys(fps, attr.keys(), build_rotation::<RotZ>);
                    timeline.key_frames.append(&mut rot_z);
                }
                _ => {}
            }
        }

        timeline
            .key_frames
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        timeline
    }

    pub(crate) fn positions(&self) -> Sampler<SamplerPrimitive<f32>> {
        let mut positions = Sampler {
            input: vec![],
            output: vec![],
            function: InterpolationFunction::Linear,
        };

        let mut current_pos = (0., 0., 0.);
        let mut current_time = None;

        for (time, (x, y, z)) in self.key_frames.iter().filter_map(|key| match key {
            &Key {
                key_frame: KeyFrame::Position { x, y, z },
                time,
                ..
            } => Some((time, (x, y, z))),
            _ => None,
        }) {
            if let Some(current_time) = current_time {
                if current_time != time {
                    positions.input.push(current_time);
                    positions.output.push(SamplerPrimitive::Vec3([
                        current_pos.0,
                        current_pos.1,
                        current_pos.2,
                    ]));
                }
            }
            current_time = time.into();
            if let Some(x) = x {
                current_pos.0 = x;
            }
            if let Some(y) = y {
                current_pos.1 = y;
            }
            if let Some(z) = z {
                current_pos.2 = z;
            }
        }

        if let Some(current_time) = current_time {
            positions.input.push(current_time);
            positions.output.push(SamplerPrimitive::Vec3([
                current_pos.0,
                current_pos.1,
                current_pos.2,
            ]));
        }
        positions
    }

    pub(crate) fn rotations(&self) -> Sampler<SamplerPrimitive<f32>> {
        let mut rotations = Sampler {
            input: vec![],
            output: vec![],
            function: InterpolationFunction::Linear,
        };

        let mut current_rot = (0., 0., 0.);
        let mut current_time = None;

        for (time, (x, y, z)) in self.key_frames.iter().filter_map(|key| match key {
            &Key {
                key_frame: KeyFrame::Rotation { x, y, z },
                time,
                ..
            } => Some((time, (x, y, z))),
            _ => None,
        }) {
            if let Some(current_time) = current_time {
                if current_time != time {
                    rotations.input.push(current_time);
                    rotations.output.push(SamplerPrimitive::Vec3([
                        current_rot.0,
                        current_rot.1,
                        current_rot.2,
                    ]));
                }
            }
            current_time = time.into();
            if let Some(x) = x {
                current_rot.0 = x;
            }
            if let Some(y) = y {
                current_rot.1 = y;
            }
            if let Some(z) = z {
                current_rot.2 = z;
            }
        }

        if let Some(current_time) = current_time {
            rotations.input.push(current_time);
            rotations.output.push(SamplerPrimitive::Vec3([
                current_rot.0,
                current_rot.1,
                current_rot.2,
            ]));
        }
        rotations
    }

    pub(crate) fn sprite_renders(&self, data: &SpriteStudioData) -> Sampler<SpriteRenderPrimitive> {
        let mut renders = Sampler {
            input: vec![],
            output: vec![],
            function: InterpolationFunction::Linear,
        };
        let mut current_index = 0;
        let mut current_time = None;

        for (time, idx) in self.key_frames.iter().filter_map(|key| match key {
            Key {
                key_frame: KeyFrame::Cell { map_id, part_name },
                time,
                ..
            } => data
                .cell_index(*map_id as usize, part_name)
                .map(|idx| (*time, idx)),
            _ => None,
        }) {
            if let Some(current_time) = current_time {
                if current_time != time {
                    renders.input.push(current_time);
                    renders
                        .output
                        .push(SpriteRenderPrimitive::SpriteIndex(current_index));
                }
            }
            current_time = time.into();
            current_index = idx;
        }

        if let Some(current_time) = current_time {
            renders.input.push(current_time);
            renders
                .output
                .push(SpriteRenderPrimitive::SpriteIndex(current_index));
        }

        renders
    }

    pub(crate) fn sprite_render(&self, data: &SpriteStudioData) -> Option<SpriteRenderPrefab> {
        self.key_frames.iter().find_map(|key| match key {
            Key {
                key_frame: KeyFrame::Cell { map_id, part_name },
                ..
            } => data.cell_maps().nth(*map_id as usize).and_then(|cell_map| {
                cell_map.cells().enumerate().find_map(|(idx, cell)| {
                    if cell.name() == part_name {
                        let mut render = SpriteRenderPrefab::default();
                        render.sheet = SpriteSheetReference::Name(cell_map.name().into()).into();
                        render.sprite_number = idx;
                        render.into()
                    } else {
                        None
                    }
                })
            }),
            _ => None,
        })
    }
}

fn to_keys<'a, I, U, B, F>(fps: u32, values: I, fold_fn: F) -> Vec<Key<U>>
where
    I: Iterator<Item = &'a KeyValue>,
    B: Default + Into<KeyFrame<U>>,
    F: FnMut(B, &ValueType) -> B + Clone,
    U: From<i32>,
{
    let mut keys = vec![];

    for v in values {
        let time = v.time() as f32 / fps as f32;
        let interpolation = v.interpolation();
        let key_frame = v.values().fold(B::default(), fold_fn.clone()).into();
        let key = Key {
            time,
            key_frame,
            interpolation,
        };
        keys.push(key);
    }

    keys
}
