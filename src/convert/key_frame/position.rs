use super::KeyFrame;
use sprite_studio::*;

pub(crate) struct PosX;
pub(crate) struct PosY;
pub(crate) struct PosZ;

pub(crate) struct PositionBuilder<T> {
    position: Option<f32>,
    p_type: std::marker::PhantomData<T>,
}

pub(crate) fn build_position<T>(
    mut builder: PositionBuilder<T>,
    v: &ValueType,
) -> PositionBuilder<T> {
    match v {
        &ValueType::Simple(v) => builder.position = v.into(),
        _ => {}
    }
    builder
}

impl<T> Default for PositionBuilder<T> {
    fn default() -> Self {
        PositionBuilder {
            position: None,
            p_type: std::marker::PhantomData,
        }
    }
}

impl<U> Into<KeyFrame<U>> for PositionBuilder<PosX> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Position {
            x: self.position,
            y: None,
            z: None,
        }
    }
}

impl<U> Into<KeyFrame<U>> for PositionBuilder<PosY> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Position {
            y: self.position,
            x: None,
            z: None,
        }
    }
}
impl<U> Into<KeyFrame<U>> for PositionBuilder<PosZ> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Position {
            z: self.position,
            y: None,
            x: None,
        }
    }
}
