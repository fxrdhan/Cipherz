TARGET = block_cipher

all:
	$(MAKE) -C c TARGET=../$(TARGET)

clean:
	$(MAKE) -C c clean TARGET=../$(TARGET)

.PHONY: all clean
