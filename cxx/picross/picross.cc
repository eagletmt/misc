#include <iostream>
#include <sstream>
#include <vector>
#include <algorithm>
using namespace std;

enum status {
  UNKNOWN,
  BLACK,
  WHITE,
};

struct contradict {};

void show_row(const vector<status> &row, int C) {
  for (int i = 0; i < C; i++) {
    if (i != 0 && i % 5 == 0) {
      cout << '|';
    }
    switch (row[i]) {
      case UNKNOWN:
        cout << ' ';
        break;
      case BLACK:
        cout << 'o';
        break;
      case WHITE:
        cout << 'x';
        break;
    }
  }
  cout << endl;
}

void show_grid(const vector<vector<status>> &grid, int R, int C) {
  const string rule(C + (C / 5 - 1), '-');
  for (int i = 0; i < R; i++) {
    if (i != 0 && i % 5 == 0) {
      cout << rule << endl;
    }
    show_row(grid[i], C);
  }
  cout << endl;
}

void all_patterns_aux(vector<vector<status>> &patterns,
                      const vector<int> &hints, int i, vector<status> &pat,
                      int j) {
  const int N = pat.size();
  const int M = hints.size();
  if (i == M) {
    patterns.push_back(pat);
  } else {
    for (int k = j; k + hints[i] <= N; k++) {
      for (int l = 0; l < hints[i]; l++) {
        pat[k + l] = BLACK;
      }
      all_patterns_aux(patterns, hints, i + 1, pat, k + hints[i] + 1);
      for (int l = 0; l < hints[i]; l++) {
        pat[k + l] = WHITE;
      }
    }
  }
}

void all_patterns(vector<vector<status>> &patterns, const vector<int> &hints,
                  int N) {
  vector<status> pat(N, WHITE);
  all_patterns_aux(patterns, hints, 0, pat, 0);
}

void transpose(vector<vector<status>> &grid) {
  const int N = grid.size();
  for (int i = 0; i < N; i++) {
    for (int j = i + 1; j < N; j++) {
      swap(grid[i][j], grid[j][i]);
    }
  }
}

bool bad_pattern_p(const vector<status> &row, const vector<status> &pattern) {
  const int N = row.size();
  for (int i = 0; i < N; i++) {
    if (row[i] != UNKNOWN && row[i] != pattern[i]) {
      return true;
    }
  }
  return false;
}

void erase_bad_patterns(const vector<status> &row,
                        vector<vector<status>> &patterns) {
  auto it = remove_if(patterns.begin(), patterns.end(),
                      [&row](const vector<status> &pattern) {
                        return bad_pattern_p(row, pattern);
                      });
  patterns.erase(it, patterns.end());
}

bool deduce_one(vector<status> &row, vector<vector<status>> &patterns, int N) {
  vector<status> candidate(N, UNKNOWN);
  vector<int> bad_candidate(N, 0);

  erase_bad_patterns(row, patterns);
  for (const vector<status> &pattern : patterns) {
    for (int i = 0; i < N; i++) {
      if (row[i] == UNKNOWN && !bad_candidate[i]) {
        if (candidate[i] == UNKNOWN) {
          candidate[i] = pattern[i];
        } else if (candidate[i] == pattern[i]) {
          // Good
        } else {
          // Bad
          bad_candidate[i] = true;
        }
      }
    }
  }
  if (count(row.begin(), row.begin() + N, UNKNOWN) != 0 && patterns.empty()) {
    throw contradict();
  }

  bool modified = false;
  for (int i = 0; i < N; i++) {
    if (row[i] == UNKNOWN && candidate[i] != UNKNOWN && !bad_candidate[i]) {
      row[i] = candidate[i];
      modified = true;
    }
  }
  return modified;
}

bool deduce(vector<vector<status>> &grid,
            vector<vector<vector<status>>> &patterns, int R, int C) {
  for (int i = 0; i < R; i++) {
    if (deduce_one(grid[i], patterns[i], C)) {
      return true;
    }
  }
  return false;
}

vector<vector<status>> solve(const vector<vector<int>> &row_hints,
                             const vector<vector<int>> &col_hints) {
  const int R = row_hints.size();
  const int C = col_hints.size();
  const int N = std::max(R, C);
  vector<vector<status>> grid(N, vector<status>(N, UNKNOWN));

  vector<vector<vector<status>>> row_patterns(R), col_patterns(C);
  for (int i = 0; i < R; i++) {
    all_patterns(row_patterns[i], row_hints[i], C);
  }
  for (int i = 0; i < C; i++) {
    all_patterns(col_patterns[i], col_hints[i], R);
  }

  for (;;) {
    const bool row_modified = deduce(grid, row_patterns, R, C);
    if (row_modified) {
      show_grid(grid, R, C);
    }
    transpose(grid);
    const bool col_modified = deduce(grid, col_patterns, C, R);
    transpose(grid);
    if (col_modified) {
      show_grid(grid, R, C);
    }
    if (!row_modified && !col_modified) {
      break;
    }
  }

  return grid;
}

int main() try {
  vector<string> lines;
  for (string line; getline(cin, line);) {
    lines.push_back(line);
  }
  unsigned R, C;
  {
    istringstream iss(lines[0]);
    iss >> R >> C;
  }
  if (lines.size() != R + C + 1) {
    cerr << "Invalid input format" << endl;
  }
  vector<vector<int>> row_hints(R), col_hints(C);
  for (unsigned i = 0; i < R; i++) {
    istringstream iss(lines[i + 1]);
    for (int n; iss >> n;) {
      row_hints[i].push_back(n);
    }
  }
  for (unsigned i = 0; i < C; i++) {
    istringstream iss(lines[i + 1 + R]);
    for (int n; iss >> n;) {
      col_hints[i].push_back(n);
    }
  }

  solve(row_hints, col_hints);

  return 0;
} catch (const contradict &) {
  cerr << "Invalid input: contradict" << endl;
}
