# robinson

| type    | pre | position | children | final main (after calc children) | cross (after calc as child)
| -------- | ------- | ------- | ------- | ------- | ------- |
| block  | width -> (specific, auto by container, auto)    | left, top | recursive -> vertical(block, anonymous block) | height -> (specific, auto by sum) | width -> (auto cross fill by max) |
| anonymous block | width -> (auto by container, auto) | left, top | recursive -> vertical(line) | height -> (auto by sum) | width -> (auto cross fill by max) |
| line | width -> (auto by container, auto) | left, top | recursive -> horizontal(inline, inline-block) | width -> (auto by sum and limit by context) | height -> (auto cross fill by max) |
| inline run    | width -> (auto by measure) | left, top | / | width -> (limit with dynamic break and create new line by context) | top -> (auto cross fix) |
| inline-block    | width -> (specific, auto) | left, top | recursive -> vertical(block, anonymous block) | height -> (specific, auto by sum) | width -> (auto cross fill by max), top -> (auto cross fix) |

## second

| type           | pre                                                                 | place children                          | self main                              | self cross                    | fill children                             | fixing children #[cfg(not(margin-auto))]         |
|----------------|---------------------------------------------------------------------|-----------------------------------------|----------------------------------------|------------------------------|-------------------------------------------|---------------------------------------------------|
| block          | width -> (specific, auto take one line from container, empty auto) | recursive -> vertical(block, anonymous block) | height -> (specific, auto by children sum) | width -> (specific, max by children) | width -> block (empty auto -> fill one line) | left -> child-leading(block, anonymous block)      |
| anonymous block | width -> (empty auto)                                              | recursive -> horizontal(inline run, inline block)           | width -> (auto by children sum but limit by context) | height -> lines Σ (max by children) | /                                         | top -> child-baseline(inline run, inline-block)   |
| inline run     | measurable (width, height)                                         | /                                       | /                                      | /                             | /                                         | /                                                 |
| inline-block   | measurable (width calc from container, height calc from container)  | recursive -> vertical(block, anonymous block) | height -> (specific, auto by children sum) | width -> (specific, max by children) | width -> block (empty auto -> fill one line) | left -> child-leading(block, anonymous block)      |