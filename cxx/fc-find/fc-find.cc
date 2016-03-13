#include <iostream>
#include <regex>
#include <vector>
#include <algorithm>
#include <cstring>
#include <fontconfig/fontconfig.h>

int main(int argc, char *argv[]) {
  if (argc < 2) {
    std::cerr << "Usage: " << argv[0] << " STRING" << std::endl;
    return 1;
  }

  FcInit();

  std::vector<FcChar32> targets;
  std::regex re("^U\\+([0-9a-fA-F]+)$", std::regex::extended);
  for (int i = 1; i < argc; ++i) {
    int len = std::strlen(argv[i]);
    std::cmatch m;
    if (std::regex_match(argv[i], m, re)) {
      targets.push_back(std::stoi(m[1], 0, 16));
    } else {
      const FcChar8 *str = reinterpret_cast<const FcChar8 *>(argv[i]);
      FcChar32 c = 0;
      int n;
      while ((n = FcUtf8ToUcs4(str, &c, len)) != 0) {
        targets.push_back(c);
        str += n;
        len -= n;
      }
    }
  }

  static const FcChar8 name[] = { 0 };
  std::unique_ptr<FcPattern, decltype(&FcPatternDestroy)> p(FcNameParse(name), FcPatternDestroy);
  FcResult result;
  FcConfigSubstitute(NULL, p.get(), FcMatchPattern);
  FcDefaultSubstitute(p.get());
  std::unique_ptr<FcFontSet, decltype(&FcFontSetDestroy)> set(
      FcFontSort(NULL, p.get(), FcFalse, NULL, &result), FcFontSetDestroy);
  if (result != FcResultMatch) {
    std::cerr << "FcFontSort failed: result=" << result << std::endl;
    return 1;
  }

  for (int i = 0; i < set->nfont; ++i) {
    const FcPattern *font = set->fonts[i];
    FcChar8 *fullname = nullptr;
    FcCharSet *charset = nullptr;

    if (FcPatternGetString(font, FC_FULLNAME, 0, &fullname) != FcResultMatch) {
      continue;
    }
    if (FcPatternGetCharSet(font, FC_CHARSET, 0, &charset) != FcResultMatch) {
      continue;
    }
    if (std::all_of(targets.begin(), targets.end(), [charset](FcChar32 c) { return FcCharSetHasChar(charset, c) == FcTrue; })) {
      std::cout << fullname << std::endl;
    }
  }

  return 0;
}
