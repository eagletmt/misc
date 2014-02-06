CC = g++
CFLAGS = -Wall -W -O2 `pkg-config --cflags poppler-glib poppler cairo`
LDFLAGS = `pkg-config --libs poppler-glib poppler cairo`
OBJ = pdf-unlock.o
TARGET = pdf-unlock

$(TARGET): $(OBJ)
	$(CC) $(LDFLAGS) -o $@ $(OBJ)

clean:
	$(RM) $(OBJ) $(TARGET)
