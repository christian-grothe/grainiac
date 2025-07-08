#pragma once
#include "Arduino.h"
#include "components.h"

struct Instance{
    int midichannel;
    int muxA;
    int muxB;

    Pot pots_a[8]{
        Pot(20),
        Pot(21),
        Pot(22),
        Pot(23),
        Pot(24),
        Pot(25),
        Pot(26),
        Pot(27),
    };

    Pot pots_b[4]{
        Pot(28),
        Pot(29),
        Pot(30),
        Pot(31),
    };

    Btn btns[4]{
        Btn(32),
        Btn(33),
        Btn(34),
        Btn(35),
    };
};
