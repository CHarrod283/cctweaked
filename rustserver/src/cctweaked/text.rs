
/*
|0 1 2 3 4 5 6 7 8 9 A B C D E F
-+--------------------------------
0|  ☺ ☻ ♥ ♦ ♣ ♠ ● ○     ♂ ♀   ♪ ♬
1|▶ ◀ ↕ ‼ ¶ ░ ▬ ↨ ⬆ ⬇ ➡ ⬅ ∟ ⧺ ▲ ▼
2|  ! " # $ % & ' ( ) * + , - . /
3|0 1 2 3 4 5 6 7 8 9 : ; < = > ?
4|@ A B C D E F G H I J K L M N O
5|P Q R S T U V W X Y Z [ \ ] ^ _
6|` a b c d e f g h i j k l m n o
7|p q r s t u v w x y z { | } ~ ▒
8|⠀ ⠁ ⠈ ⠉ ⠂ ⠃ ⠊ ⠋ ⠐ ⠑ ⠘ ⠙ ⠒ ⠓ ⠚ ⠛
9|⠄ ⠅ ⠌ ⠍ ⠆ ⠇ ⠎ ⠏ ⠔ ⠕ ⠜ ⠝ ⠖ ⠗ ⠞ ⠟
A|▓ ¡ ¢ £ ¤ ¥ ¦ █ ¨ © ª « ¬ ­ ® ¯
B|° ± ² ³ ´ µ ¶ · ¸ ¹ º » ¼ ½ ¾ ¿
C|À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î Ï
D|Ð Ñ Ò Ó Ô Õ Ö × Ø Ù Ú Û Ü Ý Þ ß
E|à á â ã ä å æ ç è é ê ë ì í î ï
F|ð ñ ò ó ô õ ö ÷ ø ù ú û ü ý þ ÿ
*/
use ratatui::symbols::border;
use thiserror::Error;

fn is_valid_cctweaked_char(c: char) -> bool {
    // Check if the character is a valid CCTweaked character
    c.is_ascii() && (c as u8) < 0x80
}

/// Convert a Unicode character to its equivalent cctweaked byte value
/// according to the CCTweaked character set. <br>
/// ```raw
///  |0 1 2 3 4 5 6 7 8 9 A B C D E F 
/// -+-------------------------------- 
/// 0|  ☺ ☻ ♥ ♦ ♣ ♠ ● ○     ♂ ♀   ♪ ♬ 
/// 1|▶ ◀ ↕ ‼ ¶ ░ ▬ ↨ ⬆ ⬇ ➡ ⬅ ∟ ⧺ ▲ ▼ 
/// 2|  ! " # $ % & ' ( ) * + , - . / 
/// 3|0 1 2 3 4 5 6 7 8 9 : ; < = > ? 
/// 4|@ A B C D E F G H I J K L M N O
/// 5|P Q R S T U V W X Y Z [ \ ] ^ _
/// 6|` a b c d e f g h i j k l m n o
/// 7|p q r s t u v w x y z { | } ~ ▒
/// 8|⠀ ⠁ ⠈ ⠉ ⠂ ⠃ ⠊ ⠋ ⠐ ⠑ ⠘ ⠙ ⠒ ⠓ ⠚ ⠛
/// 9|⠄ ⠅ ⠌ ⠍ ⠆ ⠇ ⠎ ⠏ ⠔ ⠕ ⠜ ⠝ ⠖ ⠗ ⠞ ⠟
/// A|▓ ¡ ¢ £ ¤ ¥ ¦ █ ¨ © ª « ¬ ­ ® ¯
/// B|° ± ² ³ ´ µ ¶ · ¸ ¹ º » ¼ ½ ¾ ¿
/// C|À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î Ï
/// D|Ð Ñ Ò Ó Ô Õ Ö × Ø Ù Ú Û Ü Ý Þ ß
/// E|à á â ã ä å æ ç è é ê ë ì í î ï
/// F|ð ñ ò ó ô õ ö ÷ ø ù ú û ü ý þ ÿ
/// ```
pub fn get_cctweaked_equivalent(c: char) -> Option<u8> {
    // Map CCTweaked characters to their equivalents
    if  c.is_ascii() && (c as u8) > 0x1f && (c as u8) < 0x7f {
        return Some(c as u8)
    }
    match c {
        //  |0 1 2 3 4 5 6 7 8 9 A B C D E F 
        // -+-------------------------------- 
        // 0|  ☺ ☻ ♥ ♦ ♣ ♠ ● ○     ♂ ♀   ♪ ♬
        '☺' => Some(0x01),
        '☻' => Some(0x02),
        '♥' => Some(0x03),
        '♦' => Some(0x04),
        '♣' => Some(0x05),
        '♠' => Some(0x06),
        '●' => Some(0x07),
        '○' => Some(0x08),
        '♂' => Some(0x0b),
        '♀' => Some(0x0c),
        '♪' => Some(0x0e),
        '♬' => Some(0x0f),
        // 1|▶ ◀ ↕ ‼ ¶ ░ ▬ ↨ ⬆ ⬇ ➡ ⬅ ∟ ⧺ ▲ ▼ 
        '▶' => Some(0x10),
        '◀' => Some(0x11),
        '↕' => Some(0x12),
        '‼' => Some(0x13),
        '¶' => Some(0x14),
        '░' => Some(0x15),
        '▬' => Some(0x16),
        '↨' => Some(0x17),
        '⬆' => Some(0x18),
        '⬇' => Some(0x19),
        '➡' => Some(0x1a),
        '⬅' => Some(0x1b),
        '∟' => Some(0x1c),
        '⧺' => Some(0x1d),
        '▲' => Some(0x1e),
        '▼' => Some(0x1f),
        // ascii
        '▒' => Some(0x7f),
        // 8|⠀ ⠁ ⠈ ⠉ ⠂ ⠃ ⠊ ⠋ ⠐ ⠑ ⠘ ⠙ ⠒ ⠓ ⠚ ⠛
        '⠁' => Some(0x81),
        '⠈' => Some(0x82),
        '⠉' => Some(0x83),
        '⠂' => Some(0x84),
        '⠃' => Some(0x85),
        '⠊' => Some(0x86),
        '⠋' => Some(0x87),
        '⠐' => Some(0x88),
        '⠑' => Some(0x89),
        '⠘' => Some(0x8a),
        '⠙' => Some(0x8b),
        '⠒' => Some(0x8c),
        '⠓' => Some(0x8d),
        '⠚' => Some(0x8e),
        '⠛' => Some(0x8f),
        // 9|⠄ ⠅ ⠌ ⠍ ⠆ ⠇ ⠎ ⠏ ⠔ ⠕ ⠜ ⠝ ⠖ ⠗ ⠞ ⠟
        '⠄' => Some(0x90),
        '⠅' => Some(0x91),
        '⠌' => Some(0x92),
        '⠍' => Some(0x93),
        '⠆' => Some(0x94),
        '⠇' => Some(0x95),
        '⠎' => Some(0x96),
        '⠏' => Some(0x97),
        '⠔' => Some(0x98),
        '⠕' => Some(0x99),
        '⠜' => Some(0x9a),
        '⠝' => Some(0x9b),
        '⠖' => Some(0x9c),
        '⠗' => Some(0x9d),
        '⠞' => Some(0x9e),
        '⠟' => Some(0x9f),
        // A|▓ ¡ ¢ £ ¤ ¥ ¦ █ ¨ © ª « ¬ ­ ® ¯
        '▓' => Some(0xa0),
        '¡' => Some(0xa1),
        '¢' => Some(0xa2),
        '£' => Some(0xa3),
        '¤' => Some(0xa4),
        '¥' => Some(0xa5),
        '¦' => Some(0xa6),
        '█' => Some(0xa7),
        '¨' => Some(0xa8),
        '©' => Some(0xa9),
        'ª' => Some(0xaa),
        '«' => Some(0xab),
        '¬' => Some(0xac),
        '\u{AD}' => Some(0xad),
        '®' => Some(0xae),
        '¯' => Some(0xaf),
        // B|° ± ² ³ ´ µ ¶ · ¸ ¹ º » ¼ ½ ¾ ¿
        '°' => Some(0xb0),
        '±' => Some(0xb1),
        '²' => Some(0xb2),
        '³' => Some(0xb3),
        '´' => Some(0xb4),
        'µ' => Some(0xb5),
        //'¶' => Some(0xb6),
        '·' => Some(0xb7),
        '¸' => Some(0xb8),
        '¹' => Some(0xb9),
        'º' => Some(0xba),
        '»' => Some(0xbb),
        '¼' => Some(0xbc),
        '½' => Some(0xbd),
        '¾' => Some(0xbe),
        '¿' => Some(0xbf),
        // C|À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î Ï
        'À' => Some(0xc0),
        'Á' => Some(0xc1),
        'Â' => Some(0xc2),
        'Ã' => Some(0xc3),
        'Ä' => Some(0xc4),
        'Å' => Some(0xc5),
        'Æ' => Some(0xc6),
        'Ç' => Some(0xc7),
        'È' => Some(0xc8),
        'É' => Some(0xc9),
        'Ê' => Some(0xca),
        'Ë' => Some(0xcb),
        'Ì' => Some(0xcc),
        'Í' => Some(0xcd),
        'Î' => Some(0xce),
        'Ï' => Some(0xcf),
        // D|Ð Ñ Ò Ó Ô Õ Ö × Ø Ù Ú Û Ü Ý Þ ß
        'Ð' => Some(0xd0),
        'Ñ' => Some(0xd1),
        'Ò' => Some(0xd2),
        'Ó' => Some(0xd3),
        'Ô' => Some(0xd4),
        'Õ' => Some(0xd5),
        'Ö' => Some(0xd6),
        '×' => Some(0xd7),
        'Ø' => Some(0xd8),
        'Ù' => Some(0xd9),
        'Ú' => Some(0xda),
        'Û' => Some(0xdb),
        'Ü' => Some(0xdc),
        'Ý' => Some(0xdd),
        'Þ' => Some(0xde),
        'ß' => Some(0xdf),
        // E|à á â ã ä å æ ç è é ê ë ì í î ï
        'à' => Some(0xe0),
        'á' => Some(0xe1),
        'â' => Some(0xe2),
        'ã' => Some(0xe3),
        'ä' => Some(0xe4),
        'å' => Some(0xe5),
        'æ' => Some(0xe6),
        'ç' => Some(0xe7),
        'è' => Some(0xe8),
        'é' => Some(0xe9),
        'ê' => Some(0xea),
        'ë' => Some(0xeb),
        'ì' => Some(0xec),
        'í' => Some(0xed),
        'î' => Some(0xee),
        'ï' => Some(0xef),
        // F|ð ñ ò ó ô õ ö ÷ ø ù ú û ü ý þ ÿ
        'ð' => Some(0xf0),
        'ñ' => Some(0xf1),
        'ò' => Some(0xf2),
        'ó' => Some(0xf3),
        'ô' => Some(0xf4),
        'õ' => Some(0xf5),
        'ö' => Some(0xf6),
        '÷' => Some(0xf7),
        'ø' => Some(0xf8),
        'ù' => Some(0xf9),
        'ú' => Some(0xfa),
        'û' => Some(0xfb),
        'ü' => Some(0xfc),
        'ý' => Some(0xfd),
        'þ' => Some(0xfe),
        'ÿ' => Some(0xff),
        _ => None,
    }
}

pub fn convert_to_cctweaked(text: &str) -> Result<Vec<u8>, CCTweakedError> {
    let mut result = Vec::new();
    for c in text.chars() {
        if is_valid_cctweaked_char(c) {
            if let Some(byte) = get_cctweaked_equivalent(c) {
                result.push(byte);
            } else {
                return Err(CCTweakedError::InvalidCharacter(c));
            }
        }
    }
    Ok(result)
}

pub const CCTWEAKED_BORDER: border::Set = border::Set {
    top_left: "⠏",
    top_right: "⠇",
    bottom_left: "⠉",
    bottom_right: "⠁",
    vertical_left: "⠇",
    vertical_right: "⠇",
    horizontal_top: "⠉",
    horizontal_bottom: "⠉",
};

pub const CCTWEAKED_ASCII_BORDER: border::Set = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

#[derive(Debug, Error)]
pub enum CCTweakedError {
    #[error("Invalid character in CCTweaked text: {0}")]
    InvalidCharacter(char),
}
