#!/bin/bash

midi_in="grainiac:midi_in"
midi_out=""

readarray -t jack_ports < <(jack_lsp)

for i in ${!jack_ports[@]}; do
	port="${jack_ports[i]}"
	if [[ "$port" == *"a2j:Teensy"* && 
		"$port" == *"(capture)"* ]]; then
		midi_out="$port"
	fi
done

echo "$midi_out"
echo "$midi_in"
jack_connect "$midi_out" "$midi_in"
