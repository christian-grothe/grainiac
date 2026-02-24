pub const BRAILLE_0: char = ' ';
pub const BRAILLE_1: char = '⡀';
pub const BRAILLE_2: char = '⡄';
pub const BRAILLE_3: char = '⡆';
pub const BRAILLE_4: char = '⡇';
pub const BRAILLE_5: char = '⡏';
pub const BRAILLE_6: char = '⡟';
pub const BRAILLE_7: char = '⡿';
pub const BRAILLE_FULL: char = '⣿';

pub const STEPS_PER_CHAR: usize = 9;
pub const MAX_CHAR: usize = 8;

pub const STATES: [char; STEPS_PER_CHAR] = [
    BRAILLE_0,
    BRAILLE_1,
    BRAILLE_2,
    BRAILLE_3,
    BRAILLE_4,
    BRAILLE_5,
    BRAILLE_6,
    BRAILLE_7,
    BRAILLE_FULL,
];
