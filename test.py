import serial
ser = serial.Serial('/dev/ttyACM0', 115200, serial.EIGHTBITS, serial.PARITY_NONE, serial.STOPBITS_ONE)

s="S2RS"
encoded=s.encode('utf-8')
values=bytearray(encoded)

# values = bytearray([b'S', b'2', b'P', b'L'])
print(values)
ser.write(values)