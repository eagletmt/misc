# jrsonnet-lint
Linter for Jsonnet files.

## Features
### Detect unused variables

```
% cat a.jsonnet
local x = error 'error';

{
  x: x,
  local x = 1,
}
% cat b.jsonnet
local x = error 'error';

{
  x: x,
  local y = 1,
}
% target/debug/jrsonnet-lint a.jsonnet b.jsonnet
a.jsonnet:1:x is defined but unused
b.jsonnet:3:y is defined but unused
```

FIXME: Line number of unused variable location is inaccurate except for top-level `local` expression.
