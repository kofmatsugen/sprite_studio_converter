use failure::Fail;
use serde_json::error::Error as JsonError;

#[derive(Debug, Fail)]
pub enum ParseAnimationError {
    #[fail(display = "part \"{}\" has {}", _0, _1)]
    PartIndexError(String, i32),
    #[fail(display = "non supported fps: {}", fps)]
    NonSupportedFps { fps: u32 },
    #[fail(display = "float value not set")]
    NotSetFloatValue,
    #[fail(display = "bool value not set")]
    NotSetBoolValue,
    #[fail(display = "map id value not set")]
    NotSetMapId,
    #[fail(display = "cell name value not set")]
    NotSetCellName,
    #[fail(display = "color value not set")]
    NotSetColor,
    #[fail(display = "text value not set")]
    NotSetText,
    #[fail(display = "json deserialize error: {}, source: {}", err, source)]
    JsonDeserializeError { err: JsonError, source: String },
}
