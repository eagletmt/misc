CC = gcc
CFLAGS = -Wall -W -O2 `pkg-config --cflags libavcodec libavformat libavutil`
LDFLAGS = `pkg-config --libs libavcodec libavformat libavutil`
OBJ = clean-ts.o

clean-ts: $(OBJ)
	$(CC) $(LDFLAGS) -o $@ $(OBJ)

clean:
	$(RM) $(OBJ) clean-ts
