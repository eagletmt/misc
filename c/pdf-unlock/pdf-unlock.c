#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <poppler.h>
#include <cairo/cairo-pdf.h>

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

int main(int argc, char *argv[])
{
  if (argc != 3 && argc != 4) {
    g_print("Usage: %s in-locked.pdf out-unlocked.pdf [password]\n", argv[0]);
    return 1;
  }

  const char *inpath = argv[1];
  const char *outpath = argv[2];

  GError *err = NULL;
  int exitcode = 0;

  GFile *infile = g_file_new_for_path(inpath);
  cairo_surface_t *surface = NULL;
  cairo_t *cairo = NULL;

  PopplerDocument *doc = poppler_document_new_from_gfile(infile, argc == 3 ? NULL : argv[3], NULL, &err);
  if (doc == NULL && err->code == POPPLER_ERROR_ENCRYPTED) {
    g_clear_error(&err);

    char password[1024];
    if (argc == 3) {
      read_password(password, sizeof(password));
    } else {
      strncpy(password, argv[3], sizeof(password)-1);
      password[sizeof(password)-1] = '\0';
    }

    doc = poppler_document_new_from_gfile(infile, password, NULL, &err);
  }
  if (doc == NULL) {
    exitcode = 2;
    goto fail;
  }

  PopplerPage *page = poppler_document_get_page(doc, 0);
  double width, height;
  poppler_page_get_size(page, &width, &height);
  g_object_unref(page);
  surface = cairo_pdf_surface_create(outpath, width, height);
  if (cairo_surface_status(surface) != CAIRO_STATUS_SUCCESS) {
    g_print("Cannot create PDF surface: %gx%g: %s\n", width, height, outpath);
    exitcode = 3;
    goto fail;
  }
  cairo = cairo_create(surface);
  if (cairo_status(cairo) != CAIRO_STATUS_SUCCESS) {
    g_print("Cannot create cairo context\n");
    exitcode = 3;
    goto fail;
  }

  const int npages = poppler_document_get_n_pages(doc);
  int i;
  for (i = 0; i < npages; i++) {
    page = poppler_document_get_page(doc, i);
    poppler_page_render_for_printing(page, cairo);
    cairo_show_page(cairo);
    g_object_unref(page);
  }
  cairo_save(cairo);

fail:
  if (err != NULL) {
    g_print("%s(%d): %s\n", g_quark_to_string(err->domain), err->code, err->message);
    g_clear_error(&err);
  }
  cairo_destroy(cairo);
  cairo_surface_destroy(surface);
  g_object_unref(doc);
  g_object_unref(infile);

  return exitcode;
}
