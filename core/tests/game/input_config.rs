use mumba_core::game::input_config::InputConfig;
use mumba_core::game::installation::Edition;
use std::path::PathBuf;

#[test]
fn it_creates_a_valid_cfg() {
    assert_eq!(
        InputConfig::new(&Edition::Standard).to_string(),
        "Keyboard\r\n\
        1. \"Select\"   32\r\n\
        2. \"Exit\"     45\r\n\
        3. \"Misc\"     30\r\n\
        4. \"Menu\"     17\r\n\
        5. \"Toggle\"   16\r\n\
        6. \"Trigger\"  18\r\n\
        7. \"RotLt\"    44\r\n\
        8. \"RotRt\"    46\r\n\
        9. \"Start\"    31\r\n\
        10. \"Select\"  33\r\n\
        11. \"Up\"      200\r\n\
        12. \"Down\"    208\r\n\
        13. \"Left\"    203\r\n\
        14. \"Right\"   205\r\n\
        Joystick\r\n\
        1. \"Select\"   226\r\n\
        2. \"Exit\"     225\r\n\
        3. \"Misc\"     224\r\n\
        4. \"Menu\"     227\r\n\
        5. \"Toggle\"   228\r\n\
        6. \"Trigger\"  229\r\n\
        7. \"RotLt\"    230\r\n\
        8. \"RotRt\"    231\r\n\
        9. \"Start\"    232\r\n\
        10. \"Select\"  233\r\n\
        11. \"Up\"      252\r\n\
        12. \"Down\"    253\r\n\
        13. \"Left\"    254\r\n\
        14. \"Right\"   255\r\n"
    );
    assert_eq!(
        InputConfig::new(&Edition::Steam).to_string(),
        "Keyboard\r\n\
        1. \"Select\"   32\r\n\
        2. \"Exit\"     45\r\n\
        3. \"Misc\"     30\r\n\
        4. \"Menu\"     17\r\n\
        5. \"Toggle\"   16\r\n\
        6. \"Trigger\"  18\r\n\
        7. \"RotLt\"    44\r\n\
        8. \"RotRt\"    46\r\n\
        9. \"Start\"    31\r\n\
        10. \"Select\"  33\r\n\
        11. \"Up\"      200\r\n\
        12. \"Down\"    208\r\n\
        13. \"Left\"    203\r\n\
        14. \"Right\"   205\r\n\
        Joystick\r\n\
        1. \"Select\"   225\r\n\
        2. \"Exit\"     224\r\n\
        3. \"Misc\"     226\r\n\
        4. \"Menu\"     227\r\n\
        5. \"Toggle\"   228\r\n\
        6. \"Trigger\"  229\r\n\
        7. \"RotLt\"    232\r\n\
        8. \"RotRt\"    233\r\n\
        9. \"Start\"    230\r\n\
        10. \"Select\"  231\r\n\
        11. \"Up\"      252\r\n\
        12. \"Down\"    253\r\n\
        13. \"Left\"    254\r\n\
        14. \"Right\"   255\r\n"
    );
}

#[test]
fn it_parses_a_valid_cfg() {
    assert_eq!(
        InputConfig::from_file(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/input.cfg")
        )
        .unwrap()
        .to_string(),
        "Keyboard\r\n\
        1. \"Select\"   12\r\n\
        2. \"Exit\"     42\r\n\
        3. \"Misc\"     23\r\n\
        4. \"Menu\"     99\r\n\
        5. \"Toggle\"   2\r\n\
        6. \"Trigger\"  3\r\n\
        7. \"RotLt\"    44\r\n\
        8. \"RotRt\"    46\r\n\
        9. \"Start\"    31\r\n\
        10. \"Select\"  33\r\n\
        11. \"Up\"      255\r\n\
        12. \"Down\"    255\r\n\
        13. \"Left\"    255\r\n\
        14. \"Right\"   255\r\n\
        Joystick\r\n\
        1. \"Select\"   255\r\n\
        2. \"Exit\"     255\r\n\
        3. \"Misc\"     255\r\n\
        4. \"Menu\"     255\r\n\
        5. \"Toggle\"   255\r\n\
        6. \"Trigger\"  255\r\n\
        7. \"RotLt\"    255\r\n\
        8. \"RotRt\"    255\r\n\
        9. \"Start\"    255\r\n\
        10. \"Select\"  255\r\n\
        11. \"Up\"      255\r\n\
        12. \"Down\"    255\r\n\
        13. \"Left\"    255\r\n\
        14. \"Right\"   255\r\n"
    );
}
