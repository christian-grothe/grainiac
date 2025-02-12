#include <Arduino.h>
#include "components.h"
#include "instance.h"

const int s0 = 2;
const int s1 = 3;
const int s2 = 4;
const int CHANNELS = 8;
const int INSTANCES = 4;

Instance instances[INSTANCES] = {
  Instance{1, A0, A1},
  Instance{2, A2, A3},
  Instance{3, A4, A5},
  Instance{4, A6, A7},
};

void setup() {
  usbMIDI.begin();
  //Serial.begin(9600);
  pinMode(s0, OUTPUT);
  pinMode(s1, OUTPUT);
  pinMode(s2, OUTPUT);

  for(int i = 0; i < INSTANCES; i++){
    pinMode(instances[i].muxA, INPUT);
    pinMode(instances[i].muxB, INPUT);
  }

}

void loop() {
  for(int ch = 0; ch < CHANNELS; ch++){
    
    digitalWrite(s0, ch & 0x01);
    digitalWrite(s1, (ch >> 1) & 0x01);
    digitalWrite(s2, (ch >> 2) & 0x01);

    //Serial.print(analogRead(input_a));
    //Serial.print(" ");
    //Serial.print(analogRead(input_b));
    //Serial.print(" ");
    //delay(100);

    for(int i = 0; i < INSTANCES; i++){
      Instance &instance = instances[i];

      Reading reading = instance.pots_a[ch].getReading(analogRead(instance.muxA));
      if(reading.isUpdated){
        usbMIDI.sendControlChange(reading.cc, reading.val, instance.midichannel);
      }

      if(ch < 4){
        Reading reading = instance.pots_b[ch].getReading(analogRead(instance.muxB));

        if(reading.isUpdated){
         usbMIDI.sendControlChange(reading.cc, reading.val, instance.midichannel);
        }
      } else {
        Reading reading = instance.btns[ch - 4].getReading(analogRead(instance.muxB));

        if(reading.isUpdated){
         usbMIDI.sendControlChange(reading.cc, reading.val, instance.midichannel);
        }
      }

    }

  }

  //Serial.println();
} 

