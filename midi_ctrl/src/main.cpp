#include <Arduino.h>
#include "components.h"

#define MIDI_CHANNEL 1

const int s0 = 2;
const int s1 = 3;
const int s2 = 4;
const int CHANNELS = 8;
const int input_a = A0;
const int input_b = A1;

const int POT_NUM = 4;
Pot pots[POT_NUM]={
  Pot(23),
  Pot(24),
  Pot(25),
  Pot(26),
};

const int BTN_NUM = 4;
Btn btns[POT_NUM]={
  Btn(27),
  Btn(28),
  Btn(29),
  Btn(30),
};

const int KEY_NUM = 8;
Btn keys[KEY_NUM]={
  Btn(60),
  Btn(61),
  Btn(62),
  Btn(63),
  Btn(64),
  Btn(65),
  Btn(66),
  Btn(67),
};

void setup() {
  usbMIDI.begin();
  //Serial.begin(9600);
  pinMode(s0, OUTPUT);
  pinMode(s1, OUTPUT);
  pinMode(s2, OUTPUT);
  pinMode(input_a, INPUT_PULLDOWN);
  pinMode(input_b, INPUT_PULLDOWN);
  analogReadAveraging(32);
}

void loop() {
  for(int ch = 0; ch < CHANNELS; ch++){
    
    digitalWrite(s0, ch & 0x01);
    digitalWrite(s1, (ch >> 1) & 0x01);
    digitalWrite(s2, (ch >> 2) & 0x01);

    //Serial.print(analogRead(input));
    //Serial.print(" ");

    Reading reading = keys[ch].getReading(analogRead(input_b));
    if(reading.isUpdated){
      usbMIDI.sendNoteOn(reading.cc, reading.val, MIDI_CHANNEL);
    }

  if(ch < POT_NUM){
    Reading reading = pots[ch].getReading(analogRead(input_a));
    if(reading.isUpdated){
     usbMIDI.sendControlChange(reading.cc, reading.val, MIDI_CHANNEL);
    }
  }else{
    Reading reading = btns[ch - 4].getReading(analogRead(input_b));
    if(reading.isUpdated){
     usbMIDI.sendControlChange(reading.cc, reading.val, MIDI_CHANNEL);
    }
  }

  }
  //Serial.println();
} 

