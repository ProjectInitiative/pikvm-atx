#!/usr/bin/env python3
'''
Usage: ./send-atx-command.py "$COMMAND_STRING"
Where $COMMAND_STRING:
S1RS = Server 1 Reset Switch
S1PS = Server 1 Power Switch Short
S1PL = Server 1 Power Switch Long

S2RS = Server 2 Reset Switch
S2PS = Server 2 Power Switch Short
S2PL = Server 2 Power Switch Long

S3RS = Server 3 Reset Switch
S3PS = Server 3 Power Switch Short
S3PL = Server 3 Power Switch Long

S4RS = Server 4 Reset Switch
S4PS = Server 4 Power Switch Short
S4PL = Server 4 Power Switch Long
'''


import serial
import sys
ser = serial.Serial('/dev/ttyACM0', 115200, serial.EIGHTBITS, serial.PARITY_NONE, serial.STOPBITS_ONE)

s=str(sys.argv[1])
encoded=s.encode('utf-8')
values=bytearray(encoded)
print(values)
ser.write(values)