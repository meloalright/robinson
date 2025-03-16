use super::css::*;
use super::style::*;

// CSS box model. All sizes are in px.

#[derive(Clone, Copy, Default, Debug)]
pub struct Dimensions {
    // Position of the content area relative to the document origin:
    pub(crate) content: Rect,

    // Surrounding edges:
    pub(crate) padding: EdgeSizes,
    pub(crate) border: EdgeSizes,
    pub(crate) margin: EdgeSizes,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct EdgeSizes {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) top: f32,
    pub(crate) bottom: f32,
}

#[derive(Clone, Debug)]
pub struct LayoutBox<'a> {
    pub(crate) dimensions: Dimensions,
    pub(crate) box_type: BoxType<'a>,
    pub(crate) children: Vec<LayoutBox<'a>>,
}

impl Dimensions {
    // The area covered by the content area plus its padding.
    pub fn padding_box(self) -> Rect {
        self.content.expanded_by(self.padding)
    }
    // The area covered by the content area plus padding and borders.
    pub fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border.clone())
    }
    // The area covered by the content area plus padding, borders, and margin.
    pub fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

impl Rect {
    fn expanded_by(self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}
enum Display {
    Inline,
    Block,
    None,
}

impl<'a> StyledNode<'a> {
    // Return the specified value of a property if it exists, otherwise `None`.
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).map(|v| v.clone())
    }
    /// Return the specified value of property `name`, or property `fallback_name` if that doesn't
    /// exist, or value `default` if neither does.
    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }

    // The value of the `display` property (defaults to inline).
    fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }
}

// Build the tree of LayoutBoxes, but don't perform any layout calculations yet.
pub fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    // Create the root box.
    let mut root = LayoutBox::new(match style_node.display() {
        Display::Block => BoxType::BlockNode(style_node),
        Display::Inline => BoxType::InlineNode(style_node),
        Display::None => panic!("Root node has display: none."),
    });

    // Create the descendant boxes.
    for child in &style_node.children {
        match child.display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root
                .get_inline_container()
                .children
                .push(build_layout_tree(child)),
            Display::None => {} // Skip nodes with `display: none;`
        }
    }
    return root;
}

impl Value {
    pub fn is_specific_length(&self) -> bool {
        match self {
            Value::Length(_, _) => true,
            Value::Keyword(_) => false,
            Value::ColorValue(_) => false,
        }
    }

    pub fn is_auto(&self) -> bool {
        match self {
            Value::Keyword(keyword) => matches!(keyword.as_str(), "auto"),
            _ => false,
        }
    }
}

impl<'a> LayoutBox<'a> {
    // Constructor function
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(), // initially set all fields to 0.0
            children: Vec::new(),
        }
    }

    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block box has no style node"),
        }
    }
    // ...

    // Lay out a box and its descendants.
    pub fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type.clone() {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => {}  // TODO
            BoxType::AnonymousBlock => {} // TODO
        }
    }

    pub fn is_width_auto(&self) -> bool {
        let style = self.get_style_node();
        let width = style.value("width").unwrap_or(Value::Keyword("auto".to_string()));
        width.is_auto()
    }

    pub fn layout2(&mut self, container_width: Value, context_constraints_width: Value) {
        // content-box
        match self.box_type.clone() {
            BoxType::BlockNode(_) => self.layout_block2(container_width, context_constraints_width),
            BoxType::InlineNode(_) => {}  // TODO
            BoxType::AnonymousBlock => {} // TODO
        }
    }

    fn layout_block2(&mut self, container_width: Value, context_constraints_width: Value) {
        self.calculate_block(container_width.clone(), context_constraints_width.clone());
    }

    fn calculate_block(&mut self, container_width: Value, context_constraints_width: Value) {
        // 1. width -> (specific, auto take one line from container, empty auto)

        let style = self.get_style_node();
        let mut width = style.value("width").unwrap_or(Value::Keyword("auto".to_string()));

        let mut self_as_container_width = Value::Keyword("auto".to_string());
        let mut self_as_context_constraints_width = context_constraints_width.clone();

        let mut is_self_no_filled_auto = false;

        let zero = Value::Length(0.0, Unit::Px);

        if width.is_specific_length() {
            let specific_width = width.to_px();
            let underflow_content = specific_width - {
                let mut margin_left = style.lookup("margin-left", "margin", &zero);
                let mut margin_right = style.lookup("margin-right", "margin", &zero);
        
                let border_left = style.lookup("border-left-width", "border-width", &zero);
                let border_right = style.lookup("border-right-width", "border-width", &zero);
        
                let padding_left = style.lookup("padding-left", "padding", &zero);
                let padding_right = style.lookup("padding-right", "padding", &zero);
        
                sum([
                    &margin_left,
                    &margin_right,
                    &border_left,
                    &border_right,
                    &padding_left,
                    &padding_right,
                    &width,
                ]
                .iter()
                .map(|v| v.to_px()))
            };

            self.dimensions.content.width = underflow_content;
            self_as_container_width = Value::Length(underflow_content, Unit::Px);
            self_as_context_constraints_width = self_as_container_width.clone();
        } else {
            if container_width.is_specific_length() {
                let specific_container_width = container_width.to_px();

                let underflow_content = specific_container_width - {
                    let mut margin_left = style.lookup("margin-left", "margin", &zero);
                    let mut margin_right = style.lookup("margin-right", "margin", &zero);
            
                    let border_left = style.lookup("border-left-width", "border-width", &zero);
                    let border_right = style.lookup("border-right-width", "border-width", &zero);
            
                    let padding_left = style.lookup("padding-left", "padding", &zero);
                    let padding_right = style.lookup("padding-right", "padding", &zero);
            
                    sum([
                        &margin_left,
                        &margin_right,
                        &border_left,
                        &border_right,
                        &padding_left,
                        &padding_right,
                        &width,
                    ]
                    .iter()
                    .map(|v| v.to_px()))
                };
                self.dimensions.content.width = underflow_content;
                self_as_container_width = Value::Length(underflow_content, Unit::Px);
                self_as_context_constraints_width = self_as_container_width.clone();
            } else {
                is_self_no_filled_auto = true;
            }
        }

        // position
        {
            let d = &mut self.dimensions;

            // margin, border, and padding have initial value 0.
            let zero = Value::Length(0.0, Unit::Px);
    
            // If margin-top or margin-bottom is `auto`, the used value is zero.

            let mut margin_left = style.lookup("margin-left", "margin", &zero);
            let mut margin_right = style.lookup("margin-right", "margin", &zero);
    
            let border_left = style.lookup("border-left-width", "border-width", &zero);
            let border_right = style.lookup("border-right-width", "border-width", &zero);
    
            let padding_left = style.lookup("padding-left", "padding", &zero);
            let padding_right = style.lookup("padding-right", "padding", &zero);
    
            let total = sum([
                &margin_left,
                &margin_right,
                &border_left,
                &border_right,
                &padding_left,
                &padding_right,
                &width,
            ]
            .iter()
            .map(|v| v.to_px()));
    
            // If width is not auto and the total is wider than the container, treat auto margins as 0.
            if container_width.is_specific_length() {
                let container_width = container_width.to_px();
                if total > container_width {
                    if margin_left.is_auto() {
                        margin_left = Value::Length(0.0, Unit::Px);
                    }
                    if margin_right.is_auto() {
                        margin_right = Value::Length(0.0, Unit::Px);
                    }
                }
            }
    
            // Adjust used values so that the above sum equals `containing_block.width`.
            // Each arm of the `match` should increase the total width by exactly `underflow`,
            // and afterward all values should be absolute lengths in px.
            if container_width.is_specific_length() {
                let container_width = container_width.to_px();
                let underflow = container_width - total;

                match (width.is_auto(), margin_left.is_auto(), margin_right.is_auto()) {
                    // If the values are overconstrained, calculate margin_right.
                    (false, false, false) => {
                        margin_right = Value::Length(margin_right.to_px() + underflow, Unit::Px);
                    }

                    // If exactly one size is auto, its used value follows from the equality.
                    (false, false, true) => {
                        margin_right = Value::Length(underflow, Unit::Px);
                    }
                    (false, true, false) => {
                        margin_left = Value::Length(underflow, Unit::Px);
                    }

                    // If width is set to auto, any other auto values become 0.
                    (true, _, _) => {
                        if margin_left.is_auto() {
                            margin_left = Value::Length(0.0, Unit::Px);
                        }
                        if margin_right.is_auto() {
                            margin_right = Value::Length(0.0, Unit::Px);
                        }

                        if underflow >= 0.0 {
                            // Expand width to fill the underflow.
                            width = Value::Length(underflow, Unit::Px);
                        } else {
                            // Width can't be negative. Adjust the right margin instead.
                            width = Value::Length(0.0, Unit::Px);
                            margin_right = Value::Length(margin_right.to_px() + underflow, Unit::Px);
                        }
                    }

                    // If margin-left and margin-right are both auto, their used values are equal.
                    (false, true, true) => {
                        margin_left = Value::Length(underflow / 2.0, Unit::Px);
                        margin_right = Value::Length(underflow / 2.0, Unit::Px);
                    }
                }
            }
    
            let d = &mut self.dimensions;
            d.content.width = width.to_px();
    
            d.padding.left = padding_left.to_px();
            d.padding.right = padding_right.to_px();
    
            d.border.left = border_left.to_px();
            d.border.right = border_right.to_px();
    
            d.margin.left = margin_left.to_px();
            d.margin.right = margin_right.to_px();
        }
        {
            let style = self.get_style_node();
            let d = &mut self.dimensions;
    
            // margin, border, and padding have initial value 0.
            let zero = Value::Length(0.0, Unit::Px);
    
            // If margin-top or margin-bottom is `auto`, the used value is zero.
            d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
            d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();
    
            d.border.top = style
                .lookup("border-top-width", "border-width", &zero)
                .to_px();
            d.border.bottom = style
                .lookup("border-bottom-width", "border-width", &zero)
                .to_px();
    
            d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
            d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();
        }
        

        // 2. recursive -> vertical(block, anonymous block)
        let mut children_sum_height = 0f32;
        let mut children_max_width = 0f32;
        for child in &mut self.children {
            child.layout2(self_as_container_width.clone(), self_as_context_constraints_width.clone());
            child.dimensions.content.y = children_sum_height + child.dimensions.margin.top + child.dimensions.border.top + child.dimensions.padding.top;
            children_sum_height += child.dimensions.margin_box().height;
            children_max_width = children_max_width.max(child.dimensions.margin_box().width);
        }

        // 3. self main: height -> (specific, auto by children sum)
        let mut height = style.value("height").unwrap_or(Value::Keyword("auto".to_string()));
        if height.is_specific_length() {
            self.dimensions.content.height = height.to_px();
        }
        else if height.is_auto() {
            self.dimensions.content.height = children_sum_height;
        }

        // 4. self cross: width -> (specific, max by children)
        if is_self_no_filled_auto {
            self.dimensions.content.width = children_max_width;
        }

        // 5. fill children: width -> block (empty auto -> fill one line)
        if is_self_no_filled_auto {
            let underflow_content = self.dimensions.content.width - {
                let margin_left = style.lookup("margin-left", "margin", &zero);
                let margin_right = style.lookup("margin-right", "margin", &zero);
        
                let border_left = style.lookup("border-left-width", "border-width", &zero);
                let border_right = style.lookup("border-right-width", "border-width", &zero);
        
                let padding_left = style.lookup("padding-left", "padding", &zero);
                let padding_right = style.lookup("padding-right", "padding", &zero);
        
                sum([
                    &margin_left,
                    &margin_right,
                    &border_left,
                    &border_right,
                    &padding_left,
                    &padding_right,
                    &width,
                ]
                .iter()
                .map(|v| v.to_px()))
            };
            self_as_container_width = Value::Length(underflow_content, Unit::Px);
            self_as_context_constraints_width = self_as_container_width.clone();
            for child in &mut self.children {
                if child.is_width_auto() && !matches!(child.box_type, BoxType::AnonymousBlock) {
                    // auto and not anonymous -> retake one line
                    child.layout2(self_as_container_width.clone(), self_as_context_constraints_width.clone());
                }
            }
        }

        // 6. fixing children #[cfg(not(margin-auto))]
        for child in &mut self.children {
            // edge size auto by position block
            child.dimensions.content.x = child.dimensions.margin.left + child.dimensions.border.left + child.dimensions.padding.left;
        }

    }

    pub fn calc_position(&mut self) {
        for child in &mut self.children {
            child.dimensions.content.y = self.dimensions.content.y + child.dimensions.content.y;
            child.dimensions.content.x = self.dimensions.content.x + child.dimensions.content.x;
            child.calc_position();
        }
    }


    fn layout_block(&mut self, containing_block: Dimensions) {
        // Child width can depend on parent width, so we need to calculate
        // this box's width before laying out its children.
        // if no specific style width will `Auto by container limit``
        self.calculate_block_width(containing_block);

        // Determine where the box is located within its container.
        self.calculate_block_position(containing_block);

        // Recursively lay out the children of this box.
        self.layout_block_children();

        // Parent height can depend on child height, so `calculate_height`
        // must be called *after* the children are laid out.
        self.calculate_block_height();
    }

    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        // `width` has initial value `auto`.
        let auto = Value::Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());

        // margin, border, and padding have initial value 0.
        let zero = Value::Length(0.0, Unit::Px);

        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);

        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border-width", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let total = sum([
            &margin_left,
            &margin_right,
            &border_left,
            &border_right,
            &padding_left,
            &padding_right,
            &width,
        ]
        .iter()
        .map(|v| v.to_px()));

        // If width is not auto and the total is wider than the container, treat auto margins as 0.
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Value::Length(0.0, Unit::Px);
            }
            if margin_right == auto {
                margin_right = Value::Length(0.0, Unit::Px);
            }
        }

        // Adjust used values so that the above sum equals `containing_block.width`.
        // Each arm of the `match` should increase the total width by exactly `underflow`,
        // and afterward all values should be absolute lengths in px.
        let underflow = containing_block.content.width - total;

        match (width == auto, margin_left == auto, margin_right == auto) {
            // If the values are overconstrained, calculate margin_right.
            (false, false, false) => {
                margin_right = Value::Length(margin_right.to_px() + underflow, Unit::Px);
            }

            // If exactly one size is auto, its used value follows from the equality.
            (false, false, true) => {
                margin_right = Value::Length(underflow, Unit::Px);
            }
            (false, true, false) => {
                margin_left = Value::Length(underflow, Unit::Px);
            }

            // If width is set to auto, any other auto values become 0.
            (true, _, _) => {
                if margin_left == auto {
                    margin_left = Value::Length(0.0, Unit::Px);
                }
                if margin_right == auto {
                    margin_right = Value::Length(0.0, Unit::Px);
                }

                if underflow >= 0.0 {
                    // Expand width to fill the underflow.
                    width = Value::Length(underflow, Unit::Px);
                } else {
                    // Width can't be negative. Adjust the right margin instead.
                    width = Value::Length(0.0, Unit::Px);
                    margin_right = Value::Length(margin_right.to_px() + underflow, Unit::Px);
                }
            }

            // If margin-left and margin-right are both auto, their used values are equal.
            (false, true, true) => {
                margin_left = Value::Length(underflow / 2.0, Unit::Px);
                margin_right = Value::Length(underflow / 2.0, Unit::Px);
            }
        }

        let d = &mut self.dimensions;
        d.content.width = width.to_px();

        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();

        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();

        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();
    }

    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let d = &mut self.dimensions;

        // margin, border, and padding have initial value 0.
        let zero = Value::Length(0.0, Unit::Px);

        // If margin-top or margin-bottom is `auto`, the used value is zero.
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

        d.border.top = style
            .lookup("border-top-width", "border-width", &zero)
            .to_px();
        d.border.bottom = style
            .lookup("border-bottom-width", "border-width", &zero)
            .to_px();

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;

        // Position the box below all the previous boxes in the container.
        d.content.y = containing_block.content.height
            + containing_block.content.y
            + d.margin.top
            + d.border.top
            + d.padding.top;
    }

    fn layout_block_children(&mut self) {
        for child in &mut self.children {
            child.layout(self.dimensions);
            // Increment the height so each child is laid out below the previous one.
            self.dimensions.content.height += child.dimensions.margin_box().height;
        }
    }

    fn fill_block_width(&mut self) {
        todo!()
    }

    fn calculate_block_height(&mut self) {
        // If the height is set to an explicit length, use that exact length.
        // Otherwise, just keep the value set by `layout_block_children`.
        if let Some(Value::Length(h, _Px)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h;
        }
    }

    // Where a new inline child should go.
    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => self,
            BoxType::BlockNode(_) => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                match self.children.last().clone() {
                    Some(LayoutBox {
                        box_type: AnonymousBlock,
                        ..
                    }) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::AnonymousBlock)),
                }
                self.children.last_mut().unwrap()
            }
        }
    }
    // ...
}

impl<'a> LayoutBox<'a> {
    fn layout_measure_inline(&mut self) {
        // calculate measure width and height never limit by context (but if work-break=break-all need to limit and split insert to new line)

        // calculate position

        // (no children)

        // (no specific width height)
    }
}

impl<'a> LayoutBox<'a> {
    fn layout_anonymous_block(&mut self) {}

    fn calculate_anonymous_block_width(&mut self, containing_block: Dimensions) {}

    fn calculate_anonymous_block_position(&mut self, containing_block: Dimensions) {}

    fn calculate_anonymous_block_height(&mut self, containing_block: Dimensions) {}

    fn fill_anonymous_block_width(&mut self) {}
}

impl<'a> LayoutBox<'a> {
    fn layout_line(&mut self, containing_block: Dimensions) {}

    fn layout_line_width(&mut self, containing_block: Dimensions) {}

    fn layout_line_top_left(&mut self, containing_block: Dimensions) {}

    fn layout_line_children(&mut self) {}

    fn layout_line_final_width(&mut self) {}

    fn fill_line_height(&mut self) {}
}

impl<'a> LayoutBox<'a> {
    fn layout_inline_run_block(&mut self, containing_block: Dimensions) {
        self.measure_inline_run_width(containing_block);
        self.calculate_inline_run_position(containing_block);
        self.resolve_final_width_and_line_context(containing_block);
    }

    fn measure_inline_run_width(&mut self, containing_block: Dimensions) {
        todo!()
    }

    fn calculate_inline_run_position(&mut self, containing_block: Dimensions) {
        todo!()
    }

    fn resolve_final_width_and_line_context(&mut self, containing_block: Dimensions) {
        todo!()
    }

    fn fix_top(&mut self) {
        todo!()
    }
}

impl<'a> LayoutBox<'a> {
    fn layout_inline_block(&mut self, containing_block: Dimensions) {}

    fn layout_inline_block_width(&mut self, containing_block: Dimensions) {}

    fn layout_inline_block_position(&mut self, containing_block: Dimensions) {}

    fn layout_inline_block_children(&mut self) {}

    fn fill_line_block_width_and_fix_top(&mut self) {}
}

fn sum<I>(iter: I) -> f32
where
    I: Iterator<Item = f32>,
{
    iter.fold(0., |a, b| a + b)
}

mod tests {
    use super::super::css;
    use super::super::html;
    use super::*;
    #[test]
    fn test_layout() {
        let root = html::parse("<div class=\"note\"><div class=\"note\"></div></div>".to_string());
        let css =
            css::parse("div.note { display: block; margin: 20px; padding: 10px; }".to_owned());

        let styled_tree = style_tree(&root, &css);

        let mut layout_tree = build_layout_tree(&styled_tree);

        let mut dimension = Dimensions::default();

        dimension.content.width = 200.0;

        layout_tree.layout(dimension);

        println!("{:#?}", layout_tree);

        assert_eq!(layout_tree.dimensions.margin_box().width, 200.0);
        assert_eq!(layout_tree.dimensions.padding_box().width, 160.0);
        assert_eq!(layout_tree.dimensions.content.width, 140.0);

        assert_eq!(layout_tree.children[0].dimensions.margin_box().width, 140.0);
        assert_eq!(
            layout_tree.children[0].dimensions.padding_box().width,
            100.0
        );
        assert_eq!(layout_tree.children[0].dimensions.content.width, 80.0);

        assert_eq!(layout_tree.children[0].dimensions.margin_box().height, 60.0);
        assert_eq!(layout_tree.dimensions.margin_box().height, 120.0);
    }
    #[test]
    fn test_layout2() {
        let root = html::parse("<div class=\"note\"><div class=\"note\"></div></div>".to_string());
        let css =
            css::parse("div.note { display: block; margin: 20px; padding: 10px; }".to_owned());

        let styled_tree = style_tree(&root, &css);

        let mut layout_tree = build_layout_tree(&styled_tree);

        let mut dimension = Dimensions::default();

        dimension.content.width = 200.0;

        layout_tree.layout2(Value::Length(200.0, Unit::Px), Value::Length(200.0, Unit::Px));

        println!("{:#?}", layout_tree);

        assert_eq!(layout_tree.dimensions.margin_box().width, 200.0);
        assert_eq!(layout_tree.dimensions.padding_box().width, 160.0);
        assert_eq!(layout_tree.dimensions.content.width, 140.0);

        assert_eq!(layout_tree.children[0].dimensions.margin_box().width, 140.0);
        assert_eq!(
            layout_tree.children[0].dimensions.padding_box().width,
            100.0
        );
        assert_eq!(layout_tree.children[0].dimensions.content.width, 80.0);

        assert_eq!(layout_tree.children[0].dimensions.margin_box().height, 60.0);
        assert_eq!(layout_tree.dimensions.margin_box().height, 120.0);
    }
}
