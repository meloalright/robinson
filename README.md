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
| block          | width -> (specific, auto take one line from container, empty auto); edge -> (specific) | recursive -> vertical(block, anonymous block) | height -> (specific, auto by children sum) | width -> (specific, max by children) | width -> block (empty auto -> fill one line) | left -> child-leading(block, anonymous block)      |
| anonymous block | width -> (empty auto)                                              | line break recursive -> horizontal(inline run, inline block)           | width -> (auto by children sum but limit by context) | height -> lines Σ (max by children) | /                                         | top -> child-baseline(inline run, inline-block)   |
| inline run     | measurable (width, height); horizontal-edge -> (specific)                                         | /                                       | /                                      | /                             | /                                         | /                                                 |
| inline-block   | measurable (width calc from container, height calc from container); edge -> (specific)  | recursive -> vertical(block, anonymous block) | height -> (specific, auto by children sum) | width -> (specific, max by children) | width -> block (empty auto -> fill one line) | left -> child-leading(block, anonymous block)      |

* inline -> into anonymous block (build_layout_tree)

* line break segment and soft wrap opportunity break into inline formatting context and bounding (layout)


* if Text means tag
- it can literal styled (display)
- it can context styling
- it children should be content string
- (or it can contains other tag children)
- when it display block
  - it should create a new anonymous line inside
    - then split text runs
    - (or inline layout inline children tags and split)
- when it display inline
  - it should merge into one bubble inline formatting context and split runs
- when it display inline-block
  - it should create a new anonymous line inside
    - then split text runs
    - (or inline layout inline children tags and split)

* if Text means tag but macro checked (better)
- it can not check pass any tag children
- it can not check pass any children
- it can literal styled but can not check passed display not inline
- it can context styling
- (it parent if block will have create a new anonymous line -> then layout inline children tags and split it)
- (it parent will not be checked pass inline or measurable)
- (it parent if inline-block will have create a new anonymous line -> then layout inline children tags and split it)

* if Text means string
- it can not contain any tag children
- it can not have any children
- it can not literal styled (display)
- it can context styling
- (it parent if block will have create a new anonymous line -> then layout inline children tags and split it)
- (it parent if inline -> merge into deep bubble inline formatting context and split runss)
- (it parent if inline-block will have create a new anonymous line -> then layout inline children tags and split it)

* inline formatting context have to use index but not borrow