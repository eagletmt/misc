#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <memory>
#include <poppler/Error.h>
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

static void ignore(void *, ErrorCategory, Goffset, char *)
{
}

int main(int argc, char *argv[])
{
  if (argc != 3 && argc != 4) {
    g_print("Usage: %s in-locked.pdf out-unlocked.pdf [password]\n", argv[0]);
    return 1;
  }

  const char *inpath = argv[1];
  const char *outpath = argv[2];

  // Suppress poppler's "Command Line Error"
  setErrorCallback(ignore, NULL);

  const std::unique_ptr<GFile, decltype(&g_object_unref)> infile(g_file_new_for_path(inpath), g_object_unref);
  GError *err = NULL;
  std::unique_ptr<PopplerDocument, decltype(&g_object_unref)> doc(poppler_document_new_from_gfile(infile.get(), argc == 3 ? NULL : argv[3], NULL, &err), g_object_unref);
  if (!doc && err->code == POPPLER_ERROR_ENCRYPTED) {
    g_clear_error(&err);

    char password[1024];
    if (argc == 3) {
      read_password(password, sizeof(password));
    } else {
      strncpy(password, argv[3], sizeof(password)-1);
      password[sizeof(password)-1] = '\0';
    }
    doc.reset(poppler_document_new_from_gfile(infile.get(), password, NULL, &err));
  }
  if (!doc) {
    g_print("%s(%d): %s\n", g_quark_to_string(err->domain), err->code, err->message);
    g_clear_error(&err);
    return 2;
  }

  std::unique_ptr<cairo_surface_t, decltype(&cairo_surface_destroy)> surface(cairo_pdf_surface_create(outpath, 0, 0), cairo_surface_destroy);
  if (cairo_surface_status(surface.get()) != CAIRO_STATUS_SUCCESS) {
    g_print("Cannot create PDF surface: %s\n", outpath);
    return 3;
  }
  std::unique_ptr<cairo_t, decltype(&cairo_destroy)> cairo(cairo_create(surface.get()), cairo_destroy);
  if (cairo_status(cairo.get()) != CAIRO_STATUS_SUCCESS) {
    g_print("Cannot create cairo context\n");
    return 3;
  }

  const int npages = poppler_document_get_n_pages(doc.get());
  for (int i = 0; i < npages; i++) {
    PopplerPage *page = poppler_document_get_page(doc.get(), i);
    double width, height;
    poppler_page_get_size(page, &width, &height);
    cairo_pdf_surface_set_size(surface.get(), width, height);
    poppler_page_render_for_printing(page, cairo.get());
    cairo_show_page(cairo.get());
    g_object_unref(page);
  }
  cairo_save(cairo.get());

  return 0;
}
