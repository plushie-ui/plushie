use crate::protocol::TreeNode;
use serde_json::Value;

/// Prop type expectations for validation.
#[derive(Debug, Clone, Copy)]
enum PropType {
    Str,
    Number,
    Bool,
    Length,
    Color,
    Array,
    Any,
}

fn prop_type_matches(val: &Value, expected: PropType) -> bool {
    match expected {
        PropType::Str => val.is_string(),
        PropType::Number => val.is_number() || val.is_string(), // numeric strings accepted
        PropType::Bool => val.is_boolean(),
        PropType::Length => val.is_number() || val.is_string() || val.is_object(),
        PropType::Color => val.is_string(),
        PropType::Array => val.is_array(),
        PropType::Any => true,
    }
}

/// Validate props for known widget types. Only active in debug builds.
/// Logs warnings for unexpected prop names or mismatched types.
pub(crate) fn validate_props(node: &TreeNode) {
    use PropType::*;

    let expected: &[(&str, PropType)] = match node.type_name.as_str() {
        "button" => &[
            ("label", Str),
            ("style", Any),
            ("width", Length),
            ("height", Length),
            ("padding", Any),
            ("clip", Bool),
            ("disabled", Bool),
        ],
        "text" => &[
            ("content", Str),
            ("size", Number),
            ("color", Color),
            ("font", Any),
            ("width", Length),
            ("height", Length),
            ("align_x", Str),
            ("align_y", Str),
            ("line_height", Number),
            ("shaping", Str),
            ("wrapping", Str),
            ("style", Str),
        ],
        "column" => &[
            ("spacing", Number),
            ("padding", Any),
            ("width", Length),
            ("height", Length),
            ("max_width", Number),
            ("align_x", Str),
            ("clip", Bool),
            ("wrap", Bool),
        ],
        "row" => &[
            ("spacing", Number),
            ("padding", Any),
            ("width", Length),
            ("height", Length),
            ("max_width", Number),
            ("align_y", Str),
            ("clip", Bool),
            ("wrap", Bool),
        ],
        "container" => &[
            ("padding", Any),
            ("width", Length),
            ("height", Length),
            ("max_width", Number),
            ("max_height", Number),
            ("center", Bool),
            ("align_x", Str),
            ("align_y", Str),
            ("clip", Bool),
            ("style", Any),
            ("background", Any),
            ("color", Color),
            ("border", Any),
            ("shadow", Any),
        ],
        "text_input" => &[
            ("value", Str),
            ("placeholder", Str),
            ("font", Any),
            ("width", Length),
            ("padding", Any),
            ("size", Number),
            ("line_height", Number),
            ("secure", Bool),
            ("style", Any),
            ("icon", Any),
            ("disabled", Bool),
            ("id", Str),
            ("on_submit", Any),
            ("on_paste", Bool),
            ("align_x", Str),
        ],
        "slider" => &[
            ("value", Number),
            ("range", Array),
            ("step", Number),
            ("width", Length),
            ("height", Number),
            ("style", Any),
            ("shift_step", Number),
            ("default", Number),
        ],
        "checkbox" => &[
            ("label", Str),
            ("checked", Bool),
            ("size", Number),
            ("font", Any),
            ("text_size", Number),
            ("spacing", Number),
            ("width", Length),
            ("style", Any),
            ("icon", Any),
            ("disabled", Bool),
        ],
        "toggler" => &[
            ("label", Str),
            ("is_toggled", Bool),
            ("size", Number),
            ("font", Any),
            ("text_size", Number),
            ("spacing", Number),
            ("width", Length),
            ("style", Any),
            ("disabled", Bool),
        ],
        "progress_bar" => &[
            ("value", Number),
            ("range", Array),
            ("width", Length),
            ("height", Length),
            ("style", Any),
        ],
        "image" => &[
            ("source", Any),
            ("width", Length),
            ("height", Length),
            ("content_fit", Str),
            ("filter_method", Str),
            ("rotation", Any),
            ("opacity", Number),
            ("border_radius", Any),
        ],
        "svg" => &[
            ("source", Str),
            ("width", Length),
            ("height", Length),
            ("content_fit", Str),
            ("rotation", Any),
            ("opacity", Number),
            ("color", Color),
        ],
        "scrollable" => &[
            ("width", Length),
            ("height", Length),
            ("direction", Any),
            ("style", Any),
            ("anchor", Str),
            ("spacing", Number),
        ],
        "grid" => &[
            ("columns", Number),
            ("spacing", Number),
            ("width", Number),
            ("height", Number),
            ("column_width", Length),
            ("row_height", Length),
        ],
        "radio" => &[
            ("label", Str),
            ("value", Str),
            ("selected", Any),
            ("size", Number),
            ("font", Any),
            ("text_size", Number),
            ("spacing", Number),
            ("width", Length),
            ("style", Any),
            ("group", Str),
        ],
        _ => return, // Unknown widget type -- skip validation
    };

    let props = match node.props.as_object() {
        Some(p) => p,
        None => return,
    };

    let expected_names: Vec<&str> = expected.iter().map(|(name, _)| *name).collect();

    for (key, val) in props {
        match expected.iter().find(|(name, _)| name == key) {
            Some((_, expected_type)) => {
                if !prop_type_matches(val, *expected_type) {
                    log::warn!(
                        "widget '{}' ({}): prop '{}' has unexpected type {:?} (expected {:?})",
                        node.id,
                        node.type_name,
                        key,
                        val,
                        expected_type
                    );
                }
            }
            None => {
                log::warn!(
                    "widget '{}' ({}): unexpected prop '{}' (known: {:?})",
                    node.id,
                    node.type_name,
                    key,
                    expected_names
                );
            }
        }
    }
}
