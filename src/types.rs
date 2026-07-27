#[derive(Debug, Clone, PartialEq)]
pub enum GatorType {
    Plain(String),
    Gator {scheme: String, subtype: Option<String>, frames: Vec<String>},
    Unknown
}

// Parse a type string from the AST into a GatorType
// ex. "Cart3.Point<World>" -> Gator, "vec3" -> Plain, anything unrecognized -> Unknown
pub fn parse_gator_type(ty: &str) -> GatorType {
    match ty {
        "float" | "vec2" | "vec3" | "vec4" | "mat3" | "mat4" => GatorType::Plain(ty.to_string()),
        _ => {
            if let Some(lt_pos) = ty.find('<') {
                let type_part = ty[..lt_pos].trim();
                let frames_str = ty[lt_pos+1..].trim_end_matches('>');
                let frames: Vec<String> = frames_str.split(',').map(|s| s.trim().to_string()).collect();
                if let Some(dot_pos) = type_part.find('.') {
                    GatorType::Gator {
                        scheme: type_part[..dot_pos].to_string(),
                        subtype: Some(type_part[dot_pos+1..].to_string()),
                        frames
                    }
                } else {
                    GatorType::Gator {scheme: type_part.to_string(), subtype: None, frames}
                }
            } else {
                GatorType::Unknown
            }
        }
    }
}
