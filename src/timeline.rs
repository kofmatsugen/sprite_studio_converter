mod interpolate;

use amethyst_sprite_studio::timeline::{FromUser, LinearColor, TimeLine, TimeLineBuilder};
use interpolate::*;
use sprite_studio::{AttributeTag, Interpolation, KeyValue, PartAnime, ValueType};
use std::collections::BTreeMap;

pub(crate) fn part_anime_to_timeline<'de, U, P>(
    frame_count: usize,
    part_anime: &PartAnime,
    id: usize,
    parent: P,
    cell_name_dict: &Vec<BTreeMap<String, usize>>,
) -> TimeLine<U>
where
    U: FromUser + serde::Serialize + serde::Deserialize<'de>,
    P: Into<Option<usize>>,
{
    let mut builder = TimeLineBuilder::new(frame_count);

    for attr in part_anime.attributes() {
        match attr.tag() {
            AttributeTag::User => {
                append_user_keys(&mut builder, TimeLineBuilder::add_user, attr.keys())
            }
            AttributeTag::Posx => {
                append_float_keys(&mut builder, TimeLineBuilder::add_pos_x, attr.keys())
            }
            AttributeTag::Posy => {
                append_float_keys(&mut builder, TimeLineBuilder::add_pos_y, attr.keys())
            }
            AttributeTag::Posz => {
                append_float_keys(&mut builder, TimeLineBuilder::add_pos_z, attr.keys())
            }
            // AttributeTag::Prio => {
            //     append_float_keys(&mut builder, TimeLineBuilder::add_pos_z, attr.keys())
            // }
            AttributeTag::Sclx => {
                append_float_keys(&mut builder, TimeLineBuilder::add_scale_x, attr.keys())
            }
            AttributeTag::Scly => {
                append_float_keys(&mut builder, TimeLineBuilder::add_scale_y, attr.keys())
            }
            AttributeTag::Rotz => {
                append_float_keys(&mut builder, TimeLineBuilder::add_rotated, attr.keys())
            }
            AttributeTag::Hide => {
                append_bool_keys(&mut builder, TimeLineBuilder::add_visible, attr.keys())
            }
            AttributeTag::Cell => append_step_keys(
                &mut builder,
                fold_cell,
                TimeLineBuilder::add_cell,
                attr.keys(),
                cell_name_dict,
            ),
            AttributeTag::Color => append_interpolate_keys(
                &mut builder,
                fold_color,
                TimeLineBuilder::add_color,
                attr.keys(),
                &(),
            ),
            _ => {}
        }
    }

    builder.build(id, parent)
}

fn append_user_keys<'a, I, F>(builder: &mut TimeLineBuilder, add_key_fn: F, values: I)
where
    I: Iterator<Item = &'a KeyValue>,
    F: Fn(
            &mut TimeLineBuilder,
            Option<i32>,
            Option<(f32, f32)>,
            Option<(f32, f32, f32, f32)>,
            Option<String>,
        ) + Clone
        + Copy,
{
    let mut last_integer = None;
    let mut last_point = None;
    let mut last_rect = None;
    let mut last_text = None;
    let mut last_time = 0;
    for kv in values {
        let time = kv.time() as usize;
        let mut integer = None;
        let mut point = None;
        let mut rect = None;
        let mut text = None;
        for v in kv.values() {
            match v {
                &ValueType::Integer(v) => integer = v.into(),
                ValueType::Text(v) => text = v.clone().into(),
                &ValueType::Rect(x, y, w, h) => rect = (x, y, w, h).into(),
                &ValueType::Point(x, y) => point = (x, y).into(),
                _ => {}
            }
        }

        for v in (0..(time - last_time)).map(|i| {
            if i == 0 {
                (last_integer, last_point, last_rect, last_text.clone())
            } else {
                (None, None, None, None)
            }
        }) {
            add_key_fn(builder, v.0, v.1, v.2, v.3);
        }

        last_time = time;
        last_integer = integer;
        last_point = point;
        last_rect = rect;
        last_text = text;
    }

    add_key_fn(builder, last_integer, last_point, last_rect, last_text);
}

fn append_interpolate_keys<'a, I, F, F2, V, O>(
    builder: &mut TimeLineBuilder,
    fold_fn: F2,
    add_key_fn: F,
    values: I,
    option: &O,
) where
    I: Iterator<Item = &'a KeyValue>,
    F: Fn(&mut TimeLineBuilder, V) + Clone + Copy,
    F2: Fn(Option<V>, &ValueType, &O) -> Option<V> + Clone + Copy,
    V: std::ops::Mul<f32, Output = V>
        + std::ops::Add<V, Output = V>
        + Clone
        + std::fmt::Debug
        + Default,
{
    let mut last_val = None;
    let mut last_time = 0;
    let mut interpolation = Interpolation::Step;
    for kv in values {
        let time = kv.time() as usize;
        let mut val = None;
        for v in kv.values() {
            val = fold_fn(val, v, option);
        }

        match interpolation {
            Interpolation::Step => {
                for v in (0..(time - last_time)).map(|_| last_val.clone()) {
                    add_key_fn(builder, v.unwrap_or(Default::default()));
                }
            }
            Interpolation::Linear => {
                for v in lerp_to_vec(
                    last_val.clone().unwrap(),
                    val.clone().unwrap(),
                    time - last_time,
                ) {
                    add_key_fn(builder, v.into());
                }
            }
            _ => {}
        }

        last_time = time;
        last_val = val;
        interpolation = kv.interpolation();
    }

    add_key_fn(builder, last_val.unwrap_or(Default::default()));
}

fn append_step_keys<'a, I, F, F2, V, O>(
    builder: &mut TimeLineBuilder,
    fold_fn: F2,
    add_key_fn: F,
    values: I,
    option: &O,
) where
    I: Iterator<Item = &'a KeyValue>,
    F: Fn(&mut TimeLineBuilder, Option<V>) + Clone + Copy,
    F2: Fn(Option<V>, &ValueType, &O) -> Option<V> + Clone + Copy,
    V: Clone + Default,
{
    let mut last_val = None;
    let mut last_time = 0;
    for kv in values {
        let time = kv.time() as usize;
        let mut val = None;
        for v in kv.values() {
            val = fold_fn(val, v, option);
        }

        for v in (0..(time - last_time)).map(|_| last_val.clone()) {
            add_key_fn(builder, v);
        }
        last_time = time;
        last_val = val;
    }

    add_key_fn(builder, last_val);
}

fn append_bool_keys<'a, I, F>(builder: &mut TimeLineBuilder, add_key_fn: F, values: I)
where
    I: Iterator<Item = &'a KeyValue>,
    F: Fn(&mut TimeLineBuilder, bool) + Clone + Copy,
{
    let mut last_val = true;
    let mut last_time = 0;
    for kv in values {
        let time = kv.time() as usize;
        for v in kv.values() {
            match v {
                &ValueType::Simple(v) => {
                    let v = (v as u32) == 0;
                    for v in (0..(time - last_time)).map(|_| last_val) {
                        add_key_fn(builder, v);
                    }
                    last_val = v;
                }
                _ => {}
            }
        }
        last_time = time;
    }

    add_key_fn(builder, last_val.into());
}

fn append_float_keys<'a, I, F>(builder: &mut TimeLineBuilder, add_key_fn: F, values: I)
where
    I: Iterator<Item = &'a KeyValue>,
    F: Fn(&mut TimeLineBuilder, f32) + Clone + Copy,
{
    let mut last_val = 0.0;
    let mut last_time = 0;
    let mut interpolation = Interpolation::Step;
    for kv in values {
        let time = kv.time() as usize;
        let inter = kv.interpolation();
        for v in kv.values() {
            match v {
                &ValueType::Simple(v) => {
                    append_interpolate_key(
                        builder,
                        add_key_fn,
                        interpolation,
                        last_val,
                        v,
                        time - last_time,
                    );
                    last_val = v;
                }
                _ => {}
            }
        }
        last_time = time;
        interpolation = inter;
    }

    add_key_fn(builder, last_val.into());
}

fn append_interpolate_key<'a, F>(
    builder: &mut TimeLineBuilder,
    add_key_fn: F,
    interpolation: Interpolation,
    start: f32,
    end: f32,
    length: usize,
) where
    F: Fn(&mut TimeLineBuilder, f32) + Clone + Copy,
{
    let keys: Vec<Option<f32>> = match interpolation {
        Interpolation::Step => (0..)
            .map(|i| if i == 0 { Some(start) } else { None })
            .take(length)
            .collect(),
        Interpolation::Linear => lerp_to_vec(start, end, length)
            .into_iter()
            .map(Some)
            .collect(),
        _ => panic!("non supported interpolation: {:?}", interpolation),
    };

    for k in keys {
        add_key_fn(builder, k.unwrap_or_default());
    }
}

fn fold_cell(
    val: Option<(usize, usize)>,
    value_type: &ValueType,
    cell_name_dict: &Vec<BTreeMap<String, usize>>,
) -> Option<(usize, usize)> {
    // sprite studio のフォーマット的に map_id => name の順なのでひとまず問題ない...
    match value_type {
        &ValueType::MapId(map_id) => val
            .map(|(_, cell_index)| (map_id as usize, cell_index))
            .or((map_id as usize, 0).into()),
        ValueType::Name(name) => val
            .map(|(map_id, _)| (map_id, cell_name_dict[map_id][name]))
            .or((0, cell_name_dict[0][name]).into()),
        _ => val,
    }
}

fn fold_color(val: Option<LinearColor>, value_type: &ValueType, _: &()) -> Option<LinearColor> {
    // sprite studio のフォーマット的に map_id => name の順なのでひとまず問題ない...
    match value_type {
        &ValueType::Color(r, g, b, a) => Some(LinearColor(r, g, b, a)),
        _ => val,
    }
}
