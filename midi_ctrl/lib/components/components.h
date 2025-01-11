#pragma once
#include "Arduino.h"

struct Reading
{
    bool isUpdated;
    uint8_t val;
    uint8_t cc;
};

class Btn
{
public:
    Btn(uint8_t _cc): cc(_cc){}

Reading getReading(int analogIn)
{
    uint8_t newReading = map(analogIn, 0, 1023, 0, 127);
    if(!isActive && newReading > 0){
        isActive = true;
        return Reading{true, 127,cc};
    } else if (isActive && newReading == 0){
        isActive = false;
        return Reading{true, 0,cc};
    }
    return Reading{false, 0, cc};
}

private:
    bool isActive{false};
    uint8_t cc;
};


class Pot
{
public:
    Pot(uint8_t _cc): cc(_cc){}

Reading getReading(int analogIn)
{
    uint8_t newReading = map(analogIn, 0, 1023, 0, 127);
    filteredReading = alpha * newReading + (1 - alpha) * filteredReading;
    filteredReading = newReading == 127 ? newReading : filteredReading;
    Reading reading{previous != filteredReading, filteredReading, cc};
    previous = filteredReading;
    return reading;
}

private:
    float alpha{0.5f};
    uint8_t cc;
    uint8_t filteredReading{0};
    uint8_t previous{0};
};
