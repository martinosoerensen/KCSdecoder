# Example files

This folder contain test files to demonstrate how to use the tools.

After building the project, the audio recording can be decoded like this:

1. Convert the recording to 192kHz:
```
# sox examples/BLSPASCAL.wav BLSPASCAL_192kHz.wav rate 192k
```

2. Demodulate the data from the 192kHz audio file:
```
# ./target/debug/kcs_decoder --preset NASCOM BLSPASCAL_192kHz.wav
Processing 'BLSPASCAL_192kHz.wav', using output file prefix 'BLSPASCAL_192kHz'.
Active decoder config:
Channels:  All
Startbits: 1 (Space)
Databits:  8
Parity:    None
Stopbits:  1 (Mark)

Writing file 'BLSPASCAL_192kHz-ch0-00m00s-neg.dat'
Completed in 0.16 seconds, 1 files produced.
```

3. The .dat file can now be checked for completeness using the NASCOM cas format script:
```
# python3 ./NASCOM/nascom_cas_verify.py BLSPASCAL_192kHz-ch0-00m00s-neg.dat BLSPASCAL.cas
got 48 valid blocks:
address= 0x1000 , block no= 47 , data size= 256 , rawData size= 277 , checksum= 0xba
address= 0x1100 , block no= 46 , data size= 256 , rawData size= 277 , checksum= 0xb0
address= 0x1200 , block no= 45 , data size= 256 , rawData size= 277 , checksum= 0xe0
address= 0x1300 , block no= 44 , data size= 256 , rawData size= 277 , checksum= 0x1b
address= 0x1400 , block no= 43 , data size= 256 , rawData size= 277 , checksum= 0xaa
address= 0x1500 , block no= 42 , data size= 256 , rawData size= 277 , checksum= 0xb
address= 0x1600 , block no= 41 , data size= 256 , rawData size= 277 , checksum= 0x2d
address= 0x1700 , block no= 40 , data size= 256 , rawData size= 277 , checksum= 0x74
address= 0x1800 , block no= 39 , data size= 256 , rawData size= 277 , checksum= 0x1
address= 0x1900 , block no= 38 , data size= 256 , rawData size= 277 , checksum= 0x0
address= 0x1a00 , block no= 37 , data size= 256 , rawData size= 277 , checksum= 0x2f
address= 0x1b00 , block no= 36 , data size= 256 , rawData size= 277 , checksum= 0x95
address= 0x1c00 , block no= 35 , data size= 256 , rawData size= 277 , checksum= 0x3a
address= 0x1d00 , block no= 34 , data size= 256 , rawData size= 277 , checksum= 0xf9
address= 0x1e00 , block no= 33 , data size= 256 , rawData size= 277 , checksum= 0xed
address= 0x1f00 , block no= 32 , data size= 256 , rawData size= 277 , checksum= 0xf0
address= 0x2000 , block no= 31 , data size= 256 , rawData size= 277 , checksum= 0xfe
address= 0x2100 , block no= 30 , data size= 256 , rawData size= 277 , checksum= 0x66
address= 0x2200 , block no= 29 , data size= 256 , rawData size= 277 , checksum= 0x52
address= 0x2300 , block no= 28 , data size= 256 , rawData size= 277 , checksum= 0x3a
address= 0x2400 , block no= 27 , data size= 256 , rawData size= 277 , checksum= 0x7c
address= 0x2500 , block no= 26 , data size= 256 , rawData size= 277 , checksum= 0xc0
address= 0x2600 , block no= 25 , data size= 256 , rawData size= 277 , checksum= 0xd
address= 0x2700 , block no= 24 , data size= 256 , rawData size= 277 , checksum= 0x59
address= 0x2800 , block no= 23 , data size= 256 , rawData size= 277 , checksum= 0xb4
address= 0x2900 , block no= 22 , data size= 256 , rawData size= 277 , checksum= 0x84
address= 0x2a00 , block no= 21 , data size= 256 , rawData size= 277 , checksum= 0xd7
address= 0x2b00 , block no= 20 , data size= 256 , rawData size= 277 , checksum= 0xf0
address= 0x2c00 , block no= 19 , data size= 256 , rawData size= 277 , checksum= 0x76
address= 0x2d00 , block no= 18 , data size= 256 , rawData size= 277 , checksum= 0x31
address= 0x2e00 , block no= 17 , data size= 256 , rawData size= 277 , checksum= 0xac
address= 0x2f00 , block no= 16 , data size= 256 , rawData size= 277 , checksum= 0xc6
address= 0x3000 , block no= 15 , data size= 256 , rawData size= 277 , checksum= 0xe1
address= 0x3100 , block no= 14 , data size= 256 , rawData size= 277 , checksum= 0x9f
address= 0x3200 , block no= 13 , data size= 256 , rawData size= 277 , checksum= 0x13
address= 0x3300 , block no= 12 , data size= 256 , rawData size= 277 , checksum= 0xf1
address= 0x3400 , block no= 11 , data size= 256 , rawData size= 277 , checksum= 0x73
address= 0x3500 , block no= 10 , data size= 256 , rawData size= 277 , checksum= 0xc4
address= 0x3600 , block no= 9 , data size= 256 , rawData size= 277 , checksum= 0xf2
address= 0x3700 , block no= 8 , data size= 256 , rawData size= 277 , checksum= 0x9e
address= 0x3800 , block no= 7 , data size= 256 , rawData size= 277 , checksum= 0xa1
address= 0x3900 , block no= 6 , data size= 256 , rawData size= 277 , checksum= 0x8e
address= 0x3a00 , block no= 5 , data size= 256 , rawData size= 277 , checksum= 0x94
address= 0x3b00 , block no= 4 , data size= 256 , rawData size= 277 , checksum= 0xa9
address= 0x3c00 , block no= 3 , data size= 256 , rawData size= 277 , checksum= 0x39
address= 0x3d00 , block no= 2 , data size= 256 , rawData size= 277 , checksum= 0x54
address= 0x3e00 , block no= 1 , data size= 256 , rawData size= 277 , checksum= 0x93
address= 0x3f00 , block no= 0 , data size= 256 , rawData size= 277 , checksum= 0x22
all blocks appear to be accounted for
Creating output file: BLSPASCAL.cas
```

It reports success and the file `BLSPASCAL.cas` is created.
