use super::KeyFrame;
use sprite_studio::*;

#[derive(Default)]
pub(crate) struct CellBuilder {
    map_id: Option<u32>,
    part_name: Option<String>,
}

impl<U> Into<KeyFrame<U>> for CellBuilder {
    fn into(self) -> KeyFrame<U> {
        KeyFrame::Cell {
            map_id: self.map_id.expect("unset map id"),
            part_name: self.part_name.expect("unset part name"),
        }
    }
}
pub(crate) fn build_cell(mut builder: CellBuilder, v: &ValueType) -> CellBuilder {
    match v {
        &ValueType::MapId(map_id) => builder.map_id = map_id.into(),
        ValueType::Name(name) => builder.part_name = name.clone().into(),
        _ => {}
    }

    builder
}
