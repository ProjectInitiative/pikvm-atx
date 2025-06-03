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
import datetime

# --- Start of new logging section ---
LOG_FILE = '/tmp/atx_script.log'

def write_log(message):
    with open(LOG_FILE, 'a') as f:
        timestamp = datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        f.write(f'{timestamp} - {message}\n')
# --- End of new logging section ---

try:
    # Check if a command argument was provided
    if len(sys.argv) < 2:
        write_log('ERROR: Script called without any arguments.')
        sys.exit(1)

    s = str(sys.argv[1])
    write_log(f"Script started with argument: {s}")

    encoded = s.encode('utf-8')
    values = bytearray(encoded)
    write_log(f"Encoded values: {values}")

    ser = serial.Serial('/dev/pikvmatx', 115200, serial.EIGHTBITS, serial.PARITY_NONE, serial.STOPBITS_ONE, timeout=1)
    ser.write(values)
    ser.close() # It's good practice to close the serial port

    write_log("Successfully wrote to serial port.")

except Exception as e:
    # Catch any error (e.g., permission denied, serial port not found) and write it to the log
    write_log(f"AN ERROR OCCURRED: {e}")
