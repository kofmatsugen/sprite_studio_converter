use amethyst_sprite_studio::types;
use sprite_studio;

pub(crate) fn convert_part_type(part_type: &sprite_studio::PartType) -> types::PartType {
    match part_type {
        sprite_studio::PartType::Null => types::PartType::Null,
        sprite_studio::PartType::Normal => types::PartType::Normal,
        sprite_studio::PartType::Text => types::PartType::Text,
        sprite_studio::PartType::Instance => types::PartType::Instance,
    }
}

pub(crate) fn convert_bounds(bounds: &sprite_studio::BoundsType) -> Option<types::Bounds> {
    match bounds {
        sprite_studio::BoundsType::None => None,
        sprite_studio::BoundsType::Quad => Some(types::Bounds::Quad),
        sprite_studio::BoundsType::Aabb => Some(types::Bounds::Aabb),
        sprite_studio::BoundsType::Circle => Some(types::Bounds::Circle),
        sprite_studio::BoundsType::CircleMin => Some(types::Bounds::CircleMin),
        sprite_studio::BoundsType::CircleMax => Some(types::Bounds::CircleMax),
    }
}
