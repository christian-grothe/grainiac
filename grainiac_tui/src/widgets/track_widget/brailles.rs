const BRAILLE_EMPTY: char = ' ';
const BRAILLE_FULL: char = '⡇';
const BRAILLE_CENTER: char = '⠆';
const BRAILLE_SINGLE: char = '⠂';
const BRAILLE_1B: char = '⡀';
const BRAILLE_2B: char = '⡄';
const BRAILLE_3B: char = '⡆';
const BRAILLE_1T: char = '⠁';
const BRAILLE_2T: char = '⠃';
const BRAILLE_3T: char = '⠇';

const STATE_0: [char; 5] = [
    BRAILLE_EMPTY,
    BRAILLE_EMPTY,
    BRAILLE_SINGLE,
    BRAILLE_EMPTY,
    BRAILLE_EMPTY,
];
const STATE_1: [char; 5] = [
    BRAILLE_EMPTY,
    BRAILLE_EMPTY,
    BRAILLE_CENTER,
    BRAILLE_EMPTY,
    BRAILLE_EMPTY,
];
const STATE_2: [char; 5] = [
    BRAILLE_EMPTY,
    BRAILLE_EMPTY,
    BRAILLE_FULL,
    BRAILLE_EMPTY,
    BRAILLE_EMPTY,
];
const STATE_3: [char; 5] = [
    BRAILLE_EMPTY,
    BRAILLE_1B,
    BRAILLE_FULL,
    BRAILLE_1T,
    BRAILLE_EMPTY,
];
const STATE_4: [char; 5] = [
    BRAILLE_EMPTY,
    BRAILLE_2B,
    BRAILLE_FULL,
    BRAILLE_2T,
    BRAILLE_EMPTY,
];
const STATE_5: [char; 5] = [
    BRAILLE_EMPTY,
    BRAILLE_3B,
    BRAILLE_FULL,
    BRAILLE_3T,
    BRAILLE_EMPTY,
];
const STATE_6: [char; 5] = [
    BRAILLE_EMPTY,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_EMPTY,
];
const STATE_7: [char; 5] = [
    BRAILLE_1B,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_1T,
];
const STATE_8: [char; 5] = [
    BRAILLE_2B,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_2T,
];
const STATE_9: [char; 5] = [
    BRAILLE_3B,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_3T,
];
pub const STATE_10: [char; 5] = [
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_FULL,
    BRAILLE_FULL,
];

pub const NUM_STATES: usize = 11;

pub const STATES: [[char; 5]; NUM_STATES] = [
    STATE_0, STATE_1, STATE_2, STATE_3, STATE_4, STATE_5, STATE_6, STATE_7, STATE_8, STATE_9,
    STATE_10,
];
