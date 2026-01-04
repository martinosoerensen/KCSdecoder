#!/usr/bin/env python3

"""
Tool for checking data files as decoded from NASCOM tapes
Useful for verifying if a dump was good and if not, aid in reassembling the complete file by combining sources.

NASCOM tape data format:
see 'WRITE COMMAND' in http://nascomhomepage.com/pdf/Nassys3.pdf

Pilot tone - before first block:
256 bytes of 0x00

Then follows a number of blocks of up to 277 bytes each:

Each block:
10 bytes header
1-256 bytes data
1 byte checksum
10 bytes nulls

Block header:
1 byte 0x00
4 bytes 0xFF
1 byte: Load address LSB
1 byte: Load address MSB (first block is usually 0x10 here for BASIC programs)
1 byte: Size of 'data' (0x00 means 256 bytes, which is max)
1 byte: Block no / Number of blocks remaining (last block contains 0x00 here)
1 byte: Header checksum

1-256 bytes: data
1 byte: 'data' checksum (accumulate bytes with rollover)

10 bytes 0x00
"""

## Tool written by Martin O. Sørensen, Jan. 2026
import array as arr
import sys
from pathlib import Path

HEADER_LENGTH = 10
HEADERMAGICBYTES = b'\x00\xff\xff\xff\xff'

def findPilot(buffer):
    SIZE_PILOT = 256

    pilotBytesFound = 0
    for idx in range(0, len(buffer)):
        if pilotBytesFound == SIZE_PILOT:
            return idx

        if buffer[idx] == 0:
            pilotBytesFound += 1
        else:
            pilotBytesFound = 0
    return None

class TapeBlock:
    def __init__(self):
        self.rawData = None
        self.isFirstBlock = False
    
    def getLoadAddress(self):
        if self.rawData == None or len(self.rawData) < HEADER_LENGTH:
            return None
        return self.rawData[6] * 256 + self.rawData[5]

    def getBlockNo(self):
        if self.rawData == None or len(self.rawData) < HEADER_LENGTH:
            return None
        return self.rawData[8]
    
    def getData(self):
        if self.rawData == None or len(self.rawData) < HEADER_LENGTH:
            return None
        dataSize = 256 if self.rawData[7] == 0 else self.rawData[7]
        if len(self.rawData) < (HEADER_LENGTH + dataSize):
            return None
        return self.rawData[HEADER_LENGTH:HEADER_LENGTH+dataSize]
    
    # locates the first block from 'buffer'
    # if successful, returns a buffer with the remaining data,
    # otherwise returns None
    def load(self, buffer):
        self.__init__()
        postPilotByte = findPilot(buffer)
        if postPilotByte != None:
            self.isFirstBlock = True
            buffer = buffer[postPilotByte:]
        else:
            buffer = seekFirstHeader(buffer)

        if buffer == None:
            return None

        # at this point we expect 'buffer' to begin with the header
        if len(buffer) < HEADER_LENGTH:
            print("incomplete header")
            return None
        if buffer[:len(HEADERMAGICBYTES)] != HEADERMAGICBYTES:
            print("invalid header")
            return None

        checksumHeaderExpected = buffer[9]
        checksumHeaderCalculated = checksum(buffer[len(HEADERMAGICBYTES):HEADER_LENGTH - 1])
        if checksumHeaderExpected != checksumHeaderCalculated:
            print("mismatching header checksum found, expected", checksumHeaderExpected,", got", checksumHeaderCalculated)
            return None

        sizeData = 256 if buffer[7] == 0 else buffer[7]
        if len(buffer) < HEADER_LENGTH + sizeData + 1:
            print("block", buffer[8], "is incomplete")
            return None

        checksumDataExpected = buffer[HEADER_LENGTH + sizeData]
        checksumDataCalculated = checksum(buffer[HEADER_LENGTH:HEADER_LENGTH + sizeData])
        if checksumDataExpected != checksumDataCalculated:
            print("mismatching data checksum found, expected", checksumDataExpected,", got", checksumDataCalculated)
            return None

        self.rawData = buffer[:HEADER_LENGTH + sizeData + 1 + 10]
        return buffer[HEADER_LENGTH + sizeData + 1 + 10:]

def seekFirstHeader(buffer):
    idx = 0
    while True:
        if idx+len(HEADERMAGICBYTES) > len(buffer):
            return None
        if buffer[idx:idx+len(HEADERMAGICBYTES)] == HEADERMAGICBYTES:
            return buffer[idx:]
        idx += 1

def checksum(buffer):
    checksumCalculated = 0
    for val in buffer:
        checksumCalculated += val
        if checksumCalculated >= 256:
            checksumCalculated -= 256
    return checksumCalculated

def readBlocks(buffer):
    blocks = []
    while True:
        block = TapeBlock()
        newBuf = block.load(buffer)
        if newBuf == None:
            return blocks
        else:
            blocks.append(block)
            buffer = newBuf

def writeBlocksToFile(blocks : list[TapeBlock], filename : str):
    if blocks == None or filename == "":
        return None

    if blocks[0].isFirstBlock:
        file = Path(filename).open("wb")
        file.write(bytes([0] * 256))
    else:
        file = Path(filename).open("ab")
    for block in blocks:
        file.write(block.rawData)
    file.close()

if len(sys.argv) < 2:
    print("Missing input file name")
    print("Usage:\n", sys.argv[0], "<input filename> [output filename]")
    sys.exit()

data = Path(sys.argv[1]).read_bytes()
if findPilot(data) == None:
    print("Missing pilot tone")

blocks = readBlocks(data)
print("got", len(blocks), "valid blocks:")
for block in blocks:
    print("address=", hex(block.getLoadAddress()), ", block no=", block.getBlockNo(), ", data size=", len(block.getData()), ", rawData size=", len(block.rawData), ", checksum=", hex(checksum(block.getData())))

writeOutputFile = False
if len(sys.argv) > 2:
    cleanedFileName = sys.argv[2]
    writeOutputFile = True
else:
    dot = sys.argv[1].rfind(".")
    if dot == None:
        cleanedFileName = sys.argv[1] + "_cleaned"
    else:
        cleanedFileName = sys.argv[1][:dot] + "_cleaned" + sys.argv[1][dot:]

if len(blocks) == 0:
    print("found no data")
    sys.exit()

if blocks[0].isFirstBlock == True and blocks[len(blocks) - 1].getBlockNo() == 0:
    print("all blocks appear to be accounted for")
    writeOutputFile = True
if blocks[0].isFirstBlock != True:
    print("missing beginning of file")
if blocks[len(blocks) - 1].getBlockNo() != 0:
    print("missing end of file")

if (writeOutputFile):
    print("Creating output file:" if blocks[0].isFirstBlock == True else "Appending to file:", cleanedFileName)
    writeBlocksToFile(blocks, cleanedFileName)
