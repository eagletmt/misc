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

void show_row(const vector<status> &row) {
  for (const status &cell : row) {
    switch (cell) {
    case UNKNOWN:
      cout << '?';
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

void show_grid(const vector<vector<status>> &grid) {
  for (const vector<status> &row : grid) {
    show_row(row);
  }
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

bool deduce_one(vector<status> &row, vector<vector<status>> &patterns) {
  const int N = row.size();
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
            vector<vector<vector<status>>> &patterns) {
  const int N = grid.size();
  bool modified = false;
  for (int i = 0; i < N; i++) {
    if (deduce_one(grid[i], patterns[i])) {
      modified = true;
    }
  }
  return modified;
}

vector<vector<status>> solve(const vector<vector<int>> &row_hints,
                             const vector<vector<int>> &col_hints) {
  const int N = row_hints.size();
  vector<vector<status>> grid(N, vector<status>(N, UNKNOWN));

  vector<vector<vector<status>>> row_patterns(N), col_patterns(N);
  for (int i = 0; i < N; i++) {
    all_patterns(row_patterns[i], row_hints[i], N);
    all_patterns(col_patterns[i], col_hints[i], N);
  }

  for (;;) {
    const bool row_modified = deduce(grid, row_patterns);
    transpose(grid);
    const bool col_modified = deduce(grid, col_patterns);
    transpose(grid);
    if (!row_modified && !col_modified) {
      break;
    }
  }

  return grid;
}

int main() {
  vector<string> lines;
  for (string line; getline(cin, line);) {
    lines.push_back(line);
  }
  const int N = lines.size() / 2;
  vector<vector<int>> row_hints(N), col_hints(N);
  for (int i = 0; i < N; i++) {
    istringstream iss(lines[i]);
    for (int n; iss >> n;) {
      row_hints[i].push_back(n);
    }
  }
  for (int i = 0; i < N; i++) {
    istringstream iss(lines[i + N]);
    for (int n; iss >> n;) {
      col_hints[i].push_back(n);
    }
  }

  const vector<vector<status>> grid = solve(row_hints, col_hints);
  show_grid(grid);

  return 0;
}
