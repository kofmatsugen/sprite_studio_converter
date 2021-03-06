use crate::error::ParseAnimationError;
use amethyst_sprite_studio::{
    resource::{animation, data, pack, part},
    traits::animation_file::AnimationFile,
    types::{
        cell, interpolate, Bounds, EffectKey, EffectKeyBuilder, InstanceKey, InstanceKeyBuilder,
        LinearColor, PartType, VertexKey, VertexKeyBuilder,
    },
};
use std::collections::BTreeMap;
use std::str::FromStr;

const SUPPORTED_FPS: [u32; 2] = [30, 60];

pub fn convert<'a, T>(
    project: &'a sprite_studio::SpriteStudioData,
) -> Result<data::AnimationData<T>, failure::Error>
where
    T: AnimationFile,
    T::PackKey: FromStr,
    T::AnimationKey: FromStr,
    <T::PackKey as FromStr>::Err: failure::Fail,
    <T::AnimationKey as FromStr>::Err: failure::Fail,
{
    let cell_map_names = make_cell_names(&project);
    let effect_names = make_effect_names(&project);

    convert_project::<T>(project, cell_map_names, effect_names)
}

fn make_cell_names(project: &sprite_studio::SpriteStudioData) -> BTreeMap<String, Vec<String>> {
    let mut cell_map_names = BTreeMap::new();

    for cell_map in project.cell_maps() {
        // セルの指定を名前からIDに変更するための情報生成
        let mut cell_names = vec![];
        for cell in cell_map.cells() {
            cell_names.push(cell.name().to_string());
        }
        cell_map_names.insert(cell_map.file_name().to_string(), cell_names);
    }
    cell_map_names
}

fn make_effect_names(project: &sprite_studio::SpriteStudioData) -> Vec<String> {
    let mut effect_names = vec![];

    for effect in project.effects() {
        // エフェクトの指定を名前からIDに変更するための情報生成
        effect_names.push(effect.name().into());
    }
    effect_names
}

fn convert_project<'a, T>(
    project: &'a sprite_studio::SpriteStudioData,
    cell_map_names: BTreeMap<String, Vec<String>>,
    effect_names: Vec<String>,
) -> Result<data::AnimationData<T>, failure::Error>
where
    T: AnimationFile,
    T::PackKey: FromStr,
    T::AnimationKey: FromStr,
    <T::PackKey as FromStr>::Err: failure::Fail,
    <T::AnimationKey as FromStr>::Err: failure::Fail,
{
    let mut anim_packs = BTreeMap::new();
    for pack in project.packs() {
        let mut pack_cell_map_names = vec![];

        if pack.cell_map_names().count() > 0 {
            for in_pack_cell_map in pack.cell_map_names() {
                pack_cell_map_names.push(cell_map_names[in_pack_cell_map].clone());
            }
        } else {
            // アニメーション側になければもともとあるセルマップ
            for (_, cell_map) in &cell_map_names {
                pack_cell_map_names.push(cell_map.clone());
            }
        }

        log::info!("convert pack start: {}", pack.name());
        let anim_pack = convert_pack::<T>(pack, &pack_cell_map_names, &effect_names)?;
        anim_packs.insert(T::PackKey::from_str(pack.name())?, anim_pack);
    }

    Ok(data::AnimationDataBuilder::new(anim_packs).build())
}

fn convert_pack<'a, T>(
    pack: &'a sprite_studio::AnimationPack,
    cell_map_names: &Vec<Vec<String>>,
    effect_names: &Vec<String>,
) -> Result<pack::Pack<T::UserData, T::PackKey, T::AnimationKey>, failure::Error>
where
    T: AnimationFile,
    T::PackKey: FromStr,
    T::AnimationKey: FromStr,
    <T::PackKey as FromStr>::Err: failure::Fail,
    <T::AnimationKey as FromStr>::Err: failure::Fail,
{
    let mut parts = vec![];

    for part in pack.parts() {
        let (_, part) = convert_part::<T>(part, effect_names)?;

        parts.push(part);
    }

    let mut animations = BTreeMap::new();
    let mut setup = None;

    for animation in pack.animations() {
        if animation.name() == "Setup" {
            log::info!("convert animation start: {}", animation.name());
            setup = convert_animation::<T>(&parts, animation, cell_map_names)?.into();
            continue;
        } else {
            log::info!("convert animation start: {}", animation.name());
            let fps = animation.setting().fps();
            if SUPPORTED_FPS.contains(&fps) == false {
                Err(ParseAnimationError::NonSupportedFps { fps })?;
            }
            let anim = convert_animation::<T>(&parts, animation, cell_map_names)?;
            animations.insert(T::AnimationKey::from_str(animation.name())?, anim);
        }
    }

    Ok(pack::PackBuilder::new(parts, animations, setup).build())
}

fn convert_part<T>(
    part: &sprite_studio::Part,
    effect_names: &Vec<String>,
) -> Result<(u32, part::Part<T::PackKey, T::AnimationKey>), failure::Error>
where
    T: AnimationFile,
    T::PackKey: FromStr,
    T::AnimationKey: FromStr,
    <T::PackKey as FromStr>::Err: failure::Fail,
    <T::AnimationKey as FromStr>::Err: failure::Fail,
{
    let part_id = if part.index() < 0 {
        Err(ParseAnimationError::PartIndexError(
            part.name().into(),
            part.index(),
        ))
    } else {
        Ok(part.index() as u32)
    }?;

    let part_type = match part.part_type() {
        sprite_studio::PartType::Null => PartType::Null,
        sprite_studio::PartType::Normal => PartType::Normal,
        sprite_studio::PartType::Text => PartType::Text,
        sprite_studio::PartType::Instance => PartType::Instance,
        sprite_studio::PartType::Mesh => PartType::Mesh,
        sprite_studio::PartType::Bone => PartType::Bone,
        sprite_studio::PartType::Joint => PartType::Joint,
        sprite_studio::PartType::Armature => PartType::Armature,
        sprite_studio::PartType::Effect => PartType::Effect,
        sprite_studio::PartType::Mask => PartType::Mask,
    };

    let builder = part::PartBuilder::new(part.name(), part_type);

    let builder = if part.parent() < 0 {
        builder
    } else {
        builder.parent_id(part.parent() as u32)
    };

    let builder = match part.refference_animation() {
        Some((pack_name, anim_name)) => builder.refference_animation_name(
            T::PackKey::from_str(pack_name)?,
            T::AnimationKey::from_str(anim_name)?,
        ),
        _ => builder,
    };

    let bounds = match part.bounds() {
        sprite_studio::BoundsType::None => None,
        sprite_studio::BoundsType::Quad => Some(Bounds::Quad),
        sprite_studio::BoundsType::Aabb => Some(Bounds::Aabb),
        sprite_studio::BoundsType::Circle => Some(Bounds::Circle),
        sprite_studio::BoundsType::CircleMin => Some(Bounds::CircleMin),
        sprite_studio::BoundsType::CircleMax => Some(Bounds::CircleMax),
    };

    let builder = if let Some(index) = part
        .refference_effect()
        .and_then(|name| effect_names.iter().position(|n| n == name))
    {
        builder.refference_effect_index(index)
    } else {
        builder
    };

    Ok((part_id, builder.bounds(bounds).build()))
}

// SpriteStudio ではZ座標はPositionとPriorityがあるのでどっちかだけ許したい
enum PositionZType {
    Position,
    Priority,
}

fn convert_animation<T>(
    parts: &Vec<part::Part<T::PackKey, T::AnimationKey>>,
    animation: &sprite_studio::Animation,
    cell_map_names: &Vec<Vec<String>>,
) -> Result<animation::Animation<T::UserData>, ParseAnimationError>
where
    T: AnimationFile,
    T::PackKey: FromStr,
    T::AnimationKey: FromStr,
    <T::PackKey as FromStr>::Err: failure::Fail,
    <T::AnimationKey as FromStr>::Err: failure::Fail,
{
    // パーツごとにアニメーションキーフレームをまとめる
    // サポートしてるFPSか
    let fps = animation.setting().fps();

    let mut builder = animation::AnimationBuilder::new(
        parts.len(),
        animation.setting().count() as usize,
        fps as usize,
    );

    // 最初に出た方がZ座標の基準
    let mut position_z_type = None;

    for (part_id, part) in parts.iter().enumerate() {
        // パックにあるパーツと同じ名前のアニメーションデータがあるか探す
        let part_anim = animation.part_animes().find(|pa| part.name() == pa.name());
        if let Some(part_anim) = part_anim {
            for attr in part_anim.attributes() {
                for key in attr.keys() {
                    convert_key_value(
                        &mut builder,
                        part_id,
                        attr.tag(),
                        key,
                        cell_map_names,
                        &mut position_z_type,
                    )?;
                }
            }
        }
    }

    Ok(builder.build())
}

fn convert_key_value<U: serde::de::DeserializeOwned>(
    builder: &mut animation::AnimationBuilder<U>,
    part_id: usize,
    tag: &sprite_studio::AttributeTag,
    key: &sprite_studio::KeyValue,
    cell_map_names: &Vec<Vec<String>>,
    position_z_type: &mut Option<PositionZType>,
) -> Result<(), ParseAnimationError> {
    use interpolate::Interpolation;
    use sprite_studio::Interpolation as SsInter;
    let interpolation = match key.interpolation() {
        SsInter::Linear => Interpolation::Linear,
        SsInter::Hermite => Interpolation::Hermite,
        SsInter::Bezier => Interpolation::Bezier,
        SsInter::Acceleration => Interpolation::Acceleration,
        SsInter::Deceleration => Interpolation::Deceleration,
        SsInter::Step => Interpolation::Step,
    };
    let frame = key.time() as usize;
    // タグに応じてキーフレームをセットする
    match tag {
        sprite_studio::AttributeTag::Cell => {
            builder.add_cell(
                part_id,
                frame,
                interpolation,
                convert_cell(key, cell_map_names)?,
            );
        }
        sprite_studio::AttributeTag::Posx => {
            builder.add_pos_x(part_id, frame, interpolation, convert_float(key)?);
        }
        sprite_studio::AttributeTag::Posy => {
            builder.add_pos_y(part_id, frame, interpolation, convert_float(key)?);
        }
        sprite_studio::AttributeTag::Posz => {
            if let Some(PositionZType::Priority) = position_z_type {
                return Err(ParseAnimationError::ConflictPositionZ);
            } else {
                builder.add_pos_z(part_id, frame, interpolation, convert_float(key)?);
                *position_z_type = Some(PositionZType::Position);
            }
        }
        sprite_studio::AttributeTag::Rotz => {
            builder.add_rotated(part_id, frame, interpolation, convert_float(key)?);
        }
        sprite_studio::AttributeTag::Sclx => {
            builder.add_scale_x(part_id, frame, interpolation, convert_float(key)?);
        }
        sprite_studio::AttributeTag::Scly => {
            builder.add_scale_y(part_id, frame, interpolation, convert_float(key)?);
        }
        sprite_studio::AttributeTag::Alpha => {
            builder.add_alpha(part_id, frame, interpolation, convert_float(key)?);
        }
        sprite_studio::AttributeTag::Prio => {
            if let Some(PositionZType::Position) = position_z_type {
                Err(ParseAnimationError::ConflictPositionZ)?;
            } else {
                let prio = convert_float(key)?;
                builder.add_pos_z(part_id, frame, interpolation, prio);
                *position_z_type = Some(PositionZType::Priority);
            }
        }
        sprite_studio::AttributeTag::Fliph => {
            builder.add_flip_h(part_id, frame, interpolation, convert_bool(key)?)
        }
        sprite_studio::AttributeTag::Flipv => {
            builder.add_flip_v(part_id, frame, interpolation, convert_bool(key)?)
        }

        sprite_studio::AttributeTag::Hide => {
            builder.add_hide(part_id, frame, interpolation, convert_bool(key)?);
        }
        sprite_studio::AttributeTag::Color => {
            builder.add_color(part_id, frame, interpolation, convert_color(key)?);
        }
        sprite_studio::AttributeTag::Vertex => {
            builder.add_vertex(part_id, frame, interpolation, convert_vertex(key)?);
        }
        sprite_studio::AttributeTag::User => {
            builder.add_user(part_id, frame, interpolation, convert_user(key)?);
        }
        sprite_studio::AttributeTag::Instance => {
            builder.add_instance(part_id, frame, interpolation, convert_instance_key(key));
        }
        sprite_studio::AttributeTag::Effect => {
            builder.add_effect(part_id, frame, interpolation, convert_effect(key)?);
        }
        _ => {
            return Err(ParseAnimationError::NonSupportedAttribute { attribute: *tag });
        }
    }
    Ok(())
}

fn convert_cell(
    key_values: &sprite_studio::KeyValue,
    cell_map_names: &Vec<Vec<String>>,
) -> Result<cell::Cell, ParseAnimationError> {
    let map_id = key_values
        .values()
        .find_map(|v| match v {
            &sprite_studio::ValueType::MapId(id) => Some(id as usize),
            _ => None,
        })
        .ok_or(ParseAnimationError::NotSetMapId)?;
    let cell_id = key_values
        .values()
        .find_map(|v| match v {
            sprite_studio::ValueType::Name(name) => {
                cell_map_names[map_id].iter().position(|n| n == name)
            }
            _ => None,
        })
        .ok_or(ParseAnimationError::NotSetCellName)?;

    Ok(cell::CellBuilder::new(map_id, cell_id).build())
}

fn convert_float(key_values: &sprite_studio::KeyValue) -> Result<f32, ParseAnimationError> {
    key_values
        .values()
        .find_map(|v| match v {
            &sprite_studio::ValueType::Simple(val) => Some(val),
            _ => None,
        })
        .ok_or(ParseAnimationError::NotSetFloatValue)
}

fn convert_color(key_values: &sprite_studio::KeyValue) -> Result<LinearColor, ParseAnimationError> {
    key_values
        .values()
        .find_map(|v| match v {
            &sprite_studio::ValueType::Color(r, g, b, a) => Some(LinearColor(r, g, b, a)),
            _ => None,
        })
        .ok_or(ParseAnimationError::NotSetColor)
}

fn convert_bool(key_values: &sprite_studio::KeyValue) -> Result<bool, ParseAnimationError> {
    key_values
        .values()
        .find_map(|v| match v {
            &sprite_studio::ValueType::Simple(val) => Some(val as u32 != 0),
            _ => None,
        })
        .ok_or(ParseAnimationError::NotSetBoolValue)
}

fn convert_instance_key(key_values: &sprite_studio::KeyValue) -> InstanceKey {
    key_values
        .values()
        .fold(InstanceKeyBuilder::new(), |builder, v| match v {
            &sprite_studio::ValueType::LoopNum(num) => builder.loop_num(num as usize),
            &sprite_studio::ValueType::StartOffset(ofs) => builder.start_offset(ofs as usize),
            &sprite_studio::ValueType::EndOffset(ofs) => builder.end_offset(ofs as usize),
            &sprite_studio::ValueType::Infinity(inf) => builder.infinity(inf),
            &sprite_studio::ValueType::Reverse(rev) => builder.reverse(rev),
            &sprite_studio::ValueType::PingPong(ping) => builder.pingpong(ping),
            &sprite_studio::ValueType::Indipendent(independent) => builder.independent(independent),
            &sprite_studio::ValueType::Speed(speed) => builder.speed_rate(speed),
            _ => builder,
        })
        .build()
}

fn convert_user<U: serde::de::DeserializeOwned>(
    key_values: &sprite_studio::KeyValue,
) -> Result<U, ParseAnimationError> {
    let text = key_values
        .values()
        .find_map(|v| match v {
            sprite_studio::ValueType::Text(text) => Some(text.clone()),
            _ => None,
        })
        .ok_or(ParseAnimationError::NotSetText)?;

    serde_json::de::from_str(&text)
        .map_err(|err| ParseAnimationError::JsonDeserializeError { err, source: text })
}

fn convert_vertex(key_values: &sprite_studio::KeyValue) -> Result<VertexKey, ParseAnimationError> {
    let vertex = key_values
        .values()
        .fold(VertexKeyBuilder::new(), |builder, v| match v {
            &sprite_studio::ValueType::VertexLT(x, y) => builder.lt((x, y)),
            &sprite_studio::ValueType::VertexRT(x, y) => builder.rt((x, y)),
            &sprite_studio::ValueType::VertexLB(x, y) => builder.lb((x, y)),
            &sprite_studio::ValueType::VertexRB(x, y) => builder.rb((x, y)),
            _ => builder,
        })
        .build();
    Ok(vertex)
}

fn convert_effect(key_values: &sprite_studio::KeyValue) -> Result<EffectKey, ParseAnimationError> {
    let effect = key_values
        .values()
        .fold(EffectKeyBuilder::new(), |builder, v| match v {
            &sprite_studio::ValueType::StartTime(time) => builder.start_time(time),
            &sprite_studio::ValueType::Speed(speed) => builder.speed(speed),
            &sprite_studio::ValueType::Indipendent(independent) => builder.independent(independent),
            _ => builder,
        })
        .build();
    Ok(effect)
}
