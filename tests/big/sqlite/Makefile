BUILD_DIR ?= ./build
SRC_DIR ?= ./src

TARGET ?= sqlite-3.32.1.bin

CC ?= clang-7

CPPFLAGS ?= -MD -fpie
CFLAGS ?= -g -Wall -Werror
LDFLAGS ?= -g -fpie -pie -lpthread -ldl

HARDCODED_SRCS = $(wildcard $(SRC_DIR)/*.c)

SRCS := $(HARDCODED_SRCS)
OBJS := $(SRCS:$(SRC_DIR)/%.c=$(BUILD_DIR)/%.o)
DEPS := $(OBJS:.o=.d)

$(BUILD_DIR)/$(TARGET): $(OBJS)
	$(CC) $(OBJS) -o $@ $(LDFLAGS)

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c
	mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

.PHONY: clean

run: $(BUILD_DIR)/$(TARGET)
	$<

clean:
	rm -rf $(BUILD_DIR)/*


-include $(DEPS)
