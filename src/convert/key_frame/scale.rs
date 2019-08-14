use super::KeyFrame;
use sprite_studio::*;

pub(crate) struct ScaleX;
pub(crate) struct ScaleY;

pub(crate) struct ScaleBuilder<T> {
    scale: Option<f32>,
    p_type: std::marker::PhantomData<T>,
}

pub(crate) fn build_scale<T>(mut builder: ScaleBuilder<T>, v: &ValueType) -> ScaleBuilder<T> {
    match v {
        &ValueType::Simple(v) => builder.scale = v.into(),
        _ => {}
    }
    builder
}

impl<T> Default for ScaleBuilder<T> {
    fn default() -> Self {
        ScaleBuilder {
            scale: None,
            p_type: std::marker::PhantomData,
        }
    }
}

impl<U> Into<KeyFrame<U>> for ScaleBuilder<ScaleX> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Scale {
            x: self.scale,
            y: None,
        }
    }
}

impl<U> Into<KeyFrame<U>> for ScaleBuilder<ScaleY> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Scale {
            x: None,
            y: self.scale,
        }
    }
}
