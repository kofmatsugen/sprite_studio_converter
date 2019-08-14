use super::KeyFrame;
use sprite_studio::*;

#[derive(Default)]
pub(crate) struct UserBuilder<U = i32> {
    paramater: Option<U>,
}

impl<U> Into<KeyFrame<U>> for UserBuilder<U> {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::User {
            paramater: self.paramater.expect("unset user paramater"),
        }
    }
}

pub(crate) fn build_users<U: From<i32>>(
    mut builder: UserBuilder<U>,
    v: &ValueType,
) -> UserBuilder<U> {
    match v {
        &ValueType::Simple(v) => builder.paramater = U::from(v as i32).into(),
        _ => {}
    }
    builder
}
