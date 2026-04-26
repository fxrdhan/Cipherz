CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -pedantic -O2
TARGET = block_cipher
SRC = main.c

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)

clean:
	rm -f $(TARGET)

.PHONY: all clean
