#pragma once
#include "Arduino.h"
#include "components.h"

struct Instance{
    int midichannel;
    int muxA;
    int muxB;

    Pot pots_a[8]{
        Pot(1),
        Pot(2),
        Pot(3),
        Pot(4),
        Pot(5),
        Pot(6),
        Pot(7),
        Pot(8),
    };

    Pot pots_b[4]{
        Pot(9),
        Pot(10),
        Pot(11),
        Pot(12),
    };

    Btn btns[4]{
        Btn(13),
        Btn(14),
        Btn(15),
        Btn(16),
    };
};