# robinson

| type    | pre | position | children | final main (after calc children) | cross (after calc as child)
| -------- | ------- | ------- | ------- | ------- | ------- |
| block  | width -> (specific, auto by container, auto)    | left, top | recursive -> vertical(block, anonymous block) | height -> (specific, auto by sum) | width -> (auto cross fill by max) |
| anonymous block | width -> (auto by container, auto) | left, top | recursive -> vertical(line) | height -> (auto by sum) | width -> (auto cross fill by max) |
| line | width -> (auto by container, auto) | left, top | recursive -> horizontal(inline, inline-block) | width -> (auto by sum and limit by context) | height -> (auto cross fill by max) |
| inline run    | width -> (auto by measure) | left, top | / | width -> (limit with dynamic break and create new line by context) | top -> (auto cross fix) |
| inline-block    | width -> (specific, auto) | left, top | recursive -> vertical(block, anonymous block) | height -> (specific, auto by sum) | width -> (auto cross fill by max), top -> (auto cross fix) |
