//! CSS/HTML 標準色名定義
//!
//! HTML 4.01 基本16色 + CSS3 拡張色（計147色）を定義

/// 色名とHEXコードのペア
pub struct NamedColor {
    pub name: &'static str,
    pub hex: &'static str,
}

/// HTML 4.01 基本16色
pub const HTML_BASIC_COLORS: &[NamedColor] = &[
    NamedColor { name: "white", hex: "FFFFFF" },
    NamedColor { name: "silver", hex: "C0C0C0" },
    NamedColor { name: "gray", hex: "808080" },
    NamedColor { name: "black", hex: "000000" },
    NamedColor { name: "red", hex: "FF0000" },
    NamedColor { name: "maroon", hex: "800000" },
    NamedColor { name: "yellow", hex: "FFFF00" },
    NamedColor { name: "olive", hex: "808000" },
    NamedColor { name: "lime", hex: "00FF00" },
    NamedColor { name: "green", hex: "008000" },
    NamedColor { name: "aqua", hex: "00FFFF" },
    NamedColor { name: "teal", hex: "008080" },
    NamedColor { name: "blue", hex: "0000FF" },
    NamedColor { name: "navy", hex: "000080" },
    NamedColor { name: "fuchsia", hex: "FF00FF" },
    NamedColor { name: "purple", hex: "800080" },
];

/// CSS3 拡張色（HTML基本16色を含む全147色）
pub const CSS_COLORS: &[NamedColor] = &[
    // ===== 赤系 (Reds) =====
    NamedColor { name: "indianred", hex: "CD5C5C" },
    NamedColor { name: "lightcoral", hex: "F08080" },
    NamedColor { name: "salmon", hex: "FA8072" },
    NamedColor { name: "darksalmon", hex: "E9967A" },
    NamedColor { name: "lightsalmon", hex: "FFA07A" },
    NamedColor { name: "crimson", hex: "DC143C" },
    NamedColor { name: "red", hex: "FF0000" },
    NamedColor { name: "firebrick", hex: "B22222" },
    NamedColor { name: "darkred", hex: "8B0000" },

    // ===== ピンク系 (Pinks) =====
    NamedColor { name: "pink", hex: "FFC0CB" },
    NamedColor { name: "lightpink", hex: "FFB6C1" },
    NamedColor { name: "hotpink", hex: "FF69B4" },
    NamedColor { name: "deeppink", hex: "FF1493" },
    NamedColor { name: "mediumvioletred", hex: "C71585" },
    NamedColor { name: "palevioletred", hex: "DB7093" },

    // ===== オレンジ系 (Oranges) =====
    NamedColor { name: "coral", hex: "FF7F50" },
    NamedColor { name: "tomato", hex: "FF6347" },
    NamedColor { name: "orangered", hex: "FF4500" },
    NamedColor { name: "darkorange", hex: "FF8C00" },
    NamedColor { name: "orange", hex: "FFA500" },

    // ===== 黄系 (Yellows) =====
    NamedColor { name: "gold", hex: "FFD700" },
    NamedColor { name: "yellow", hex: "FFFF00" },
    NamedColor { name: "lightyellow", hex: "FFFFE0" },
    NamedColor { name: "lemonchiffon", hex: "FFFACD" },
    NamedColor { name: "lightgoldenrodyellow", hex: "FAFAD2" },
    NamedColor { name: "papayawhip", hex: "FFEFD5" },
    NamedColor { name: "moccasin", hex: "FFE4B5" },
    NamedColor { name: "peachpuff", hex: "FFDAB9" },
    NamedColor { name: "palegoldenrod", hex: "EEE8AA" },
    NamedColor { name: "khaki", hex: "F0E68C" },
    NamedColor { name: "darkkhaki", hex: "BDB76B" },

    // ===== 紫系 (Purples) =====
    NamedColor { name: "lavender", hex: "E6E6FA" },
    NamedColor { name: "thistle", hex: "D8BFD8" },
    NamedColor { name: "plum", hex: "DDA0DD" },
    NamedColor { name: "violet", hex: "EE82EE" },
    NamedColor { name: "orchid", hex: "DA70D6" },
    NamedColor { name: "fuchsia", hex: "FF00FF" },
    NamedColor { name: "magenta", hex: "FF00FF" },
    NamedColor { name: "mediumorchid", hex: "BA55D3" },
    NamedColor { name: "mediumpurple", hex: "9370DB" },
    NamedColor { name: "rebeccapurple", hex: "663399" },
    NamedColor { name: "blueviolet", hex: "8A2BE2" },
    NamedColor { name: "darkviolet", hex: "9400D3" },
    NamedColor { name: "darkorchid", hex: "9932CC" },
    NamedColor { name: "darkmagenta", hex: "8B008B" },
    NamedColor { name: "purple", hex: "800080" },
    NamedColor { name: "indigo", hex: "4B0082" },
    NamedColor { name: "slateblue", hex: "6A5ACD" },
    NamedColor { name: "darkslateblue", hex: "483D8B" },
    NamedColor { name: "mediumslateblue", hex: "7B68EE" },

    // ===== 緑系 (Greens) =====
    NamedColor { name: "greenyellow", hex: "ADFF2F" },
    NamedColor { name: "chartreuse", hex: "7FFF00" },
    NamedColor { name: "lawngreen", hex: "7CFC00" },
    NamedColor { name: "lime", hex: "00FF00" },
    NamedColor { name: "limegreen", hex: "32CD32" },
    NamedColor { name: "palegreen", hex: "98FB98" },
    NamedColor { name: "lightgreen", hex: "90EE90" },
    NamedColor { name: "mediumspringgreen", hex: "00FA9A" },
    NamedColor { name: "springgreen", hex: "00FF7F" },
    NamedColor { name: "mediumseagreen", hex: "3CB371" },
    NamedColor { name: "seagreen", hex: "2E8B57" },
    NamedColor { name: "forestgreen", hex: "228B22" },
    NamedColor { name: "green", hex: "008000" },
    NamedColor { name: "darkgreen", hex: "006400" },
    NamedColor { name: "yellowgreen", hex: "9ACD32" },
    NamedColor { name: "olivedrab", hex: "6B8E23" },
    NamedColor { name: "olive", hex: "808000" },
    NamedColor { name: "darkolivegreen", hex: "556B2F" },
    NamedColor { name: "mediumaquamarine", hex: "66CDAA" },
    NamedColor { name: "darkseagreen", hex: "8FBC8F" },
    NamedColor { name: "lightseagreen", hex: "20B2AA" },
    NamedColor { name: "darkcyan", hex: "008B8B" },
    NamedColor { name: "teal", hex: "008080" },

    // ===== 青/シアン系 (Blues/Cyans) =====
    NamedColor { name: "aqua", hex: "00FFFF" },
    NamedColor { name: "cyan", hex: "00FFFF" },
    NamedColor { name: "lightcyan", hex: "E0FFFF" },
    NamedColor { name: "paleturquoise", hex: "AFEEEE" },
    NamedColor { name: "aquamarine", hex: "7FFFD4" },
    NamedColor { name: "turquoise", hex: "40E0D0" },
    NamedColor { name: "mediumturquoise", hex: "48D1CC" },
    NamedColor { name: "darkturquoise", hex: "00CED1" },
    NamedColor { name: "cadetblue", hex: "5F9EA0" },
    NamedColor { name: "steelblue", hex: "4682B4" },
    NamedColor { name: "lightsteelblue", hex: "B0C4DE" },
    NamedColor { name: "powderblue", hex: "B0E0E6" },
    NamedColor { name: "lightblue", hex: "ADD8E6" },
    NamedColor { name: "skyblue", hex: "87CEEB" },
    NamedColor { name: "lightskyblue", hex: "87CEFA" },
    NamedColor { name: "deepskyblue", hex: "00BFFF" },
    NamedColor { name: "dodgerblue", hex: "1E90FF" },
    NamedColor { name: "cornflowerblue", hex: "6495ED" },
    NamedColor { name: "royalblue", hex: "4169E1" },
    NamedColor { name: "blue", hex: "0000FF" },
    NamedColor { name: "mediumblue", hex: "0000CD" },
    NamedColor { name: "darkblue", hex: "00008B" },
    NamedColor { name: "navy", hex: "000080" },
    NamedColor { name: "midnightblue", hex: "191970" },

    // ===== 茶系 (Browns) =====
    NamedColor { name: "cornsilk", hex: "FFF8DC" },
    NamedColor { name: "blanchedalmond", hex: "FFEBCD" },
    NamedColor { name: "bisque", hex: "FFE4C4" },
    NamedColor { name: "navajowhite", hex: "FFDEAD" },
    NamedColor { name: "wheat", hex: "F5DEB3" },
    NamedColor { name: "burlywood", hex: "DEB887" },
    NamedColor { name: "tan", hex: "D2B48C" },
    NamedColor { name: "rosybrown", hex: "BC8F8F" },
    NamedColor { name: "sandybrown", hex: "F4A460" },
    NamedColor { name: "goldenrod", hex: "DAA520" },
    NamedColor { name: "darkgoldenrod", hex: "B8860B" },
    NamedColor { name: "peru", hex: "CD853F" },
    NamedColor { name: "chocolate", hex: "D2691E" },
    NamedColor { name: "saddlebrown", hex: "8B4513" },
    NamedColor { name: "sienna", hex: "A0522D" },
    NamedColor { name: "brown", hex: "A52A2A" },
    NamedColor { name: "maroon", hex: "800000" },

    // ===== 白系 (Whites) =====
    NamedColor { name: "white", hex: "FFFFFF" },
    NamedColor { name: "snow", hex: "FFFAFA" },
    NamedColor { name: "honeydew", hex: "F0FFF0" },
    NamedColor { name: "mintcream", hex: "F5FFFA" },
    NamedColor { name: "azure", hex: "F0FFFF" },
    NamedColor { name: "aliceblue", hex: "F0F8FF" },
    NamedColor { name: "ghostwhite", hex: "F8F8FF" },
    NamedColor { name: "whitesmoke", hex: "F5F5F5" },
    NamedColor { name: "seashell", hex: "FFF5EE" },
    NamedColor { name: "beige", hex: "F5F5DC" },
    NamedColor { name: "oldlace", hex: "FDF5E6" },
    NamedColor { name: "floralwhite", hex: "FFFAF0" },
    NamedColor { name: "ivory", hex: "FFFFF0" },
    NamedColor { name: "antiquewhite", hex: "FAEBD7" },
    NamedColor { name: "linen", hex: "FAF0E6" },
    NamedColor { name: "lavenderblush", hex: "FFF0F5" },
    NamedColor { name: "mistyrose", hex: "FFE4E1" },

    // ===== グレー系 (Grays) =====
    NamedColor { name: "gainsboro", hex: "DCDCDC" },
    NamedColor { name: "lightgray", hex: "D3D3D3" },
    NamedColor { name: "lightgrey", hex: "D3D3D3" },
    NamedColor { name: "silver", hex: "C0C0C0" },
    NamedColor { name: "darkgray", hex: "A9A9A9" },
    NamedColor { name: "darkgrey", hex: "A9A9A9" },
    NamedColor { name: "gray", hex: "808080" },
    NamedColor { name: "grey", hex: "808080" },
    NamedColor { name: "dimgray", hex: "696969" },
    NamedColor { name: "dimgrey", hex: "696969" },
    NamedColor { name: "lightslategray", hex: "778899" },
    NamedColor { name: "lightslategrey", hex: "778899" },
    NamedColor { name: "slategray", hex: "708090" },
    NamedColor { name: "slategrey", hex: "708090" },
    NamedColor { name: "darkslategray", hex: "2F4F4F" },
    NamedColor { name: "darkslategrey", hex: "2F4F4F" },
    NamedColor { name: "black", hex: "000000" },
];

/// 色名からHEXコードを検索
/// 大文字小文字を区別しない
pub fn find_color_by_name(name: &str) -> Option<&'static str> {
    let name_lower = name.to_lowercase();
    
    // CSS3色リストから検索（HTML基本16色も含む）
    CSS_COLORS
        .iter()
        .find(|c| c.name == name_lower)
        .map(|c| c.hex)
}

/// すべての色名を取得
pub fn all_color_names() -> Vec<&'static str> {
    CSS_COLORS.iter().map(|c| c.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_basic_colors() {
        assert_eq!(find_color_by_name("red"), Some("FF0000"));
        assert_eq!(find_color_by_name("green"), Some("008000"));
        assert_eq!(find_color_by_name("blue"), Some("0000FF"));
        assert_eq!(find_color_by_name("lime"), Some("00FF00"));
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(find_color_by_name("RED"), Some("FF0000"));
        assert_eq!(find_color_by_name("Red"), Some("FF0000"));
        assert_eq!(find_color_by_name("rEd"), Some("FF0000"));
    }

    #[test]
    fn test_find_extended_colors() {
        assert_eq!(find_color_by_name("coral"), Some("FF7F50"));
        assert_eq!(find_color_by_name("salmon"), Some("FA8072"));
        assert_eq!(find_color_by_name("skyblue"), Some("87CEEB"));
    }

    #[test]
    fn test_gray_grey_variants() {
        assert_eq!(find_color_by_name("gray"), Some("808080"));
        assert_eq!(find_color_by_name("grey"), Some("808080"));
        assert_eq!(find_color_by_name("darkgray"), Some("A9A9A9"));
        assert_eq!(find_color_by_name("darkgrey"), Some("A9A9A9"));
    }

    #[test]
    fn test_unknown_color() {
        assert_eq!(find_color_by_name("notacolor"), None);
        assert_eq!(find_color_by_name(""), None);
    }

    #[test]
    fn test_all_color_names_count() {
        let names = all_color_names();
        // CSS3色は147色だが、gray/greyの重複があるため実際は少し多い
        assert!(names.len() >= 147);
    }
}

