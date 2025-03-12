# robinson

| type    | pre | position | children | sub
| -------- | ------- | ------- | ------- | ------- |
| block  | width -> (specific, auto by container)    | left, top | recursive -> vertical(block, anonymous block) | height -> (specific, auto by sum) |
| anonymous block | / | left, top | recursive -> horizontal(inline, inline-block) | width -> (auto by sum and limit by context), height -> (auto by sum)
| inline run    | / | left, top | / | width -> (auto by measure) (and limit with dynamic break by context)
| inline-block    | / | left, top | recursive -> vertical(block, anonymous block) | width -> (specific, auto by sum and limit by context), height -> (specific, auto by sum) |
