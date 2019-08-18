use super::KeyFrame;
use sprite_studio::*;

pub(crate) struct RotX;
pub(crate) struct RotY;
pub(crate) struct RotZ;

pub(crate) struct RotationBuilder<T> {
    rotation: Option<f32>,
    p_type: std::marker::PhantomData<T>,
}

pub(crate) fn build_rotation<T>(
    mut builder: RotationBuilder<T>,
    v: &ValueType,
) -> RotationBuilder<T> {
    match v {
        &ValueType::Simple(v) => builder.rotation = v.into(),
        _ => {}
    }
    builder
}

impl<T> Default for RotationBuilder<T> {
    fn default() -> Self {
        RotationBuilder {
            rotation: None,
            p_type: std::marker::PhantomData,
        }
    }
}

impl<U> Into<KeyFrame<U>> for RotationBuilder<RotX> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Rotation {
            x: self.rotation,
            y: None,
            z: None,
        }
    }
}

impl<U> Into<KeyFrame<U>> for RotationBuilder<RotY> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Rotation {
            y: self.rotation,
            x: None,
            z: None,
        }
    }
}
impl<U> Into<KeyFrame<U>> for RotationBuilder<RotZ> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Rotation {
            z: self.rotation,
            y: None,
            x: None,
        }
    }
}
