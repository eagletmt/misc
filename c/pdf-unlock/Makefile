CC = gcc
CFLAGS = -Wall -W -O2 `pkg-config --cflags poppler-glib cairo`
LDFLAGS = `pkg-config --libs poppler-glib cairo`
OBJ = pdf-unlock.o

pdf-unlock: $(OBJ)
	$(CC) $(LDFLAGS) -o $@ $(OBJ)

clean:
	$(RM) $(OBJ) clean-ts
