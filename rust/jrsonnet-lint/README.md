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
Variable { name: "x", path: "a.jsonnet", begin_offset: 0, end_offset: 52 }
Variable { name: "y", path: "b.jsonnet", begin_offset: 26, end_offset: 52 }
```

TODO: Convert `begin_offset` to line number
