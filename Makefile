CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -pedantic -O2
TARGET = block_cipher

all: $(TARGET)

$(TARGET): main.c
	$(CC) $(CFLAGS) -o $(TARGET) main.c

clean:
	rm -f $(TARGET)

.PHONY: all clean
