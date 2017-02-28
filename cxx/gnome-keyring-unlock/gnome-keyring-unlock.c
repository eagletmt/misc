#include <gnome-keyring.h>
#include <stdio.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>

static void read_password(char *buf, size_t n)
{
  struct termios save, t;

  fputs("Password: ", stdout);
  fflush(stdout);

  tcgetattr(STDIN_FILENO, &save);
  t = save;
  t.c_lflag &= ~ECHO;
  tcsetattr(STDIN_FILENO, TCSANOW, &t);
  fgets(buf, n, stdin);
  tcsetattr(STDIN_FILENO, TCSANOW, &save);

  putchar('\n');
  fflush(stdout);

  const size_t len = strlen(buf);
  if (buf[len-1] == '\n') {
    buf[len-1] = '\0';
  }
}

int main(void) {
  char password[1024];
  read_password(password, sizeof(password));
  GnomeKeyringResult result = gnome_keyring_unlock_sync(NULL, password);
  switch (result) {
    case GNOME_KEYRING_RESULT_OK:
      return 0;
    default:
      fprintf(stderr, "Unable to unlock: %s (result=%d)\n", gnome_keyring_result_to_message(result), result);
      return 1;
  }
}
