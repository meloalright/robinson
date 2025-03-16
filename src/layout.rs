use super::css::*;
use super::style::*;

// CSS box model. All sizes are in px.

#[derive(Clone, Copy, Default, Debug)]
pub struct Dimensions {
    // Position of the content area relative to the document origin:
    pub(crate) inner: InnerContent,

    // Surrounding edges:
    pub(crate) padding: EdgeSizes,
    pub(crate) border: EdgeSizes,
    pub(crate) margin: EdgeSizes,

    pub(crate) box_offset: BoxOffset,
    pub(crate) box_abs: BoxAbs,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct InnerContent {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct BoxOffset {
    pub(crate) left: f32,
    pub(crate) top: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct BoxAbs {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Dimensions {
    pub fn get_left(&self) -> f32 {
        self.box_offset.left + self.inner.left
    }

    pub fn get_top(&self) -> f32 {
        self.box_offset.top + self.inner.top
    }
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

pub enum InlineFormattingContextRun {
    TextRun(),
    Atom(),
}

impl<'a> LayoutBox<'a> {
    pub fn is_line_breakable() -> bool {
        todo!()
    }

    pub fn line_break(underflow_width: f32) -> Vec<InlineFormattingContextRun> {
        todo!()
    }
}

impl Dimensions {
    // // The area covered by the content area plus its padding.
    pub fn padding_box(self) -> Rect {
        self.abs_content_box().expanded_by(self.padding)
    }
    // The area covered by the content area plus padding and borders.
    pub fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border.clone())
    }
    // The area covered by the content area plus padding, borders, and margin.
    pub fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
    // The area covered by the content area plus its padding.
    fn abs_content_box(self) -> Rect {
        Rect {
            x: self.box_abs.x,
            y: self.box_abs.y,
            width: self.inner.width,
            height: self.inner.height,
        }
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
    InlineBlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}
enum Display {
    Inline,
    InlineBlock,
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
                "inline-block" => Display::InlineBlock,
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
        Display::InlineBlock => BoxType::InlineBlockNode(style_node),
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
            Display::InlineBlock => root
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
            BoxType::BlockNode(node) | BoxType::InlineNode(node) | BoxType::InlineBlockNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block box has no style node"),
        }
    }
    // ...

    pub fn is_width_auto(&self) -> bool {
        let style = self.get_style_node();
        let width = style.value("width").unwrap_or(Value::Keyword("auto".to_string()));
        width.is_auto()
    }

    pub fn layout(&mut self, container_width: Value, context_constraints_width: Value) {
        // content-box
        match self.box_type.clone() {
            BoxType::BlockNode(_) => self.layout_block(container_width, context_constraints_width),
            BoxType::InlineNode(_) => self.layout_inline(container_width, context_constraints_width),  // TODO
            BoxType::InlineBlockNode(_) => self.layout_inline_block(container_width, context_constraints_width),  // TODO
            BoxType::AnonymousBlock => self.layout_anonymous(container_width, context_constraints_width) // TODO
        }
    }

    fn layout_block(&mut self, container_width: Value, context_constraints_width: Value) {
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

            self.dimensions.inner.width = underflow_content;
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
                self.dimensions.inner.width = underflow_content;
                self_as_container_width = Value::Length(underflow_content, Unit::Px);
                self_as_context_constraints_width = self_as_container_width.clone();
            } else {
                is_self_no_filled_auto = true;
                let underflow_constraint = context_constraints_width.to_px() - {
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
                self_as_context_constraints_width = Value::Length(underflow_constraint, Unit::Px);
            }
        }

        // inner edge size position
        {
            let style = self.get_style_node();
            // margin, border, and padding have initial value 0.
            let zero = Value::Length(0.0, Unit::Px);

            let margin_left = style.lookup("margin-left", "margin", &zero);
            let margin_right = style.lookup("margin-right", "margin", &zero);
    
            let border_left = style.lookup("border-left-width", "border-width", &zero);
            let border_right = style.lookup("border-right-width", "border-width", &zero);
    
            let padding_left = style.lookup("padding-left", "padding", &zero);
            let padding_right = style.lookup("padding-right", "padding", &zero);
    
            let d = &mut self.dimensions;
    
            d.padding.left = padding_left.to_px();
            d.padding.right = padding_right.to_px();
    
            d.border.left = border_left.to_px();
            d.border.right = border_right.to_px();
    
            d.margin.left = margin_left.to_px();
            d.margin.right = margin_right.to_px();
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
            self.dimensions.inner.left = self.dimensions.margin.left + self.dimensions.border.left + self.dimensions.padding.left;
            self.dimensions.inner.top = self.dimensions.margin.top + self.dimensions.border.top + self.dimensions.padding.top;
        }
        

        // 2. recursive -> vertical(block, anonymous block)
        let mut children_sum_height = 0f32;
        let mut children_max_width = 0f32;
        for child in &mut self.children {
            child.layout(self_as_container_width.clone(), self_as_context_constraints_width.clone());
            child.dimensions.box_offset.top = children_sum_height;
            children_sum_height += child.dimensions.margin_box().height;
            children_max_width = children_max_width.max(child.dimensions.margin_box().width);
        }

        // 3. self main: height -> (specific, auto by children sum)
        let mut height = style.value("height").unwrap_or(Value::Keyword("auto".to_string()));
        if height.is_specific_length() {
            self.dimensions.inner.height = height.to_px();
        }
        else if height.is_auto() {
            self.dimensions.inner.height = children_sum_height;
        }

        // 4. self cross: width -> (specific, max by children)
        if is_self_no_filled_auto {
            self.dimensions.inner.width = children_max_width;
            self.dimensions.inner.width = children_max_width;
        }

        // 5. fill children: width -> block (empty auto -> fill one line)
        if is_self_no_filled_auto {
            let underflow_content = self.dimensions.inner.width - {
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
                if !matches!(child.box_type, BoxType::AnonymousBlock) && child.is_width_auto() {
                    // auto and not anonymous -> retake one line
                    child.layout(self_as_container_width.clone(), self_as_context_constraints_width.clone());
                }
            }
        }

        // 6. fixing children #[cfg(not(margin-auto))]
        for child in &mut self.children {
            // edge size auto by position block
            // self.dimensions.box_offset.left; // no margin so to do nothing
        }

    }


    fn layout_anonymous(&mut self, container_width: Value, context_constraints_width: Value) {
        // 1. width -> (empty auto)

        let self_as_container_width = Value::Keyword("auto".to_string());
        let self_as_context_constraints_width = context_constraints_width.clone();

        // 2. line break recursive -> horizontal(inline run, inline block)

        // line break into ifc and recursive ifc segments // todo!()

        let mut this_line_children_sum_width = 0f32;
        let mut this_line_children_max_height = 0f32;
        let mut computed_lines_sum_height = 0f32;
        let mut computed_lines_max_width = 0f32;

        let ifc_constraints_width = self_as_context_constraints_width.to_px();

        for child in &mut self.children {
            child.layout(self_as_container_width.clone(), self_as_context_constraints_width.clone());
            let mut next_sum_width = this_line_children_sum_width + child.dimensions.margin_box().width;

            if next_sum_width > ifc_constraints_width {
                // wrap
                computed_lines_sum_height += this_line_children_max_height;
                computed_lines_max_width = ifc_constraints_width;
                this_line_children_sum_width = 0f32;
                this_line_children_max_height = 0f32;

                // 5.top -> child-baseline(inline run, inline-block)
                // todo!()

                next_sum_width = this_line_children_sum_width + child.dimensions.margin_box().width;
            }

            // 3. width -> (auto by children sum but limit by context)
            child.dimensions.box_offset.left = this_line_children_sum_width;
            // 4. height -> lines Σ (max by children)
            child.dimensions.box_offset.top = computed_lines_sum_height;

            this_line_children_sum_width = next_sum_width;
            this_line_children_max_height = this_line_children_max_height.max(child.dimensions.margin_box().height);
        }

        // final line
        // 5.top -> child-baseline(inline run, inline-block)
        // todo!()
        computed_lines_sum_height += this_line_children_max_height;
        computed_lines_max_width = computed_lines_max_width.max(this_line_children_sum_width);

        self.dimensions.inner.width = computed_lines_max_width;
        self.dimensions.inner.height = computed_lines_sum_height;

        drop(this_line_children_max_height);
        drop(this_line_children_sum_width);
    }

    pub fn layout_inline(&mut self, container_width: Value, context_constraints_width: Value) {
        let style = self.get_style_node();
        let zero = Value::Length(0.0, Unit::Px);
        let width = style.lookup("measure-width", "ms-width", &zero);
        let height = style.lookup("measure-height", "ms-height", &zero);
        self.dimensions.inner.width = width.to_px();
        self.dimensions.inner.height = height.to_px();
    }

    pub fn layout_inline_block(&mut self, container_width: Value, context_constraints_width: Value) {
        // 1. measurable (width calc from container, height calc from container)

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

            self.dimensions.inner.width = underflow_content;
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
                self.dimensions.inner.width = underflow_content;
                self_as_container_width = Value::Length(underflow_content, Unit::Px);
                self_as_context_constraints_width = self_as_container_width.clone();
            } else {
                is_self_no_filled_auto = true;
                let underflow_constraint = context_constraints_width.to_px() - {
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
                self_as_context_constraints_width = Value::Length(underflow_constraint, Unit::Px);
            }
        }

        // inner edge size position
        {
            let style = self.get_style_node();
            // margin, border, and padding have initial value 0.
            let zero = Value::Length(0.0, Unit::Px);

            let margin_left = style.lookup("margin-left", "margin", &zero);
            let margin_right = style.lookup("margin-right", "margin", &zero);
    
            let border_left = style.lookup("border-left-width", "border-width", &zero);
            let border_right = style.lookup("border-right-width", "border-width", &zero);
    
            let padding_left = style.lookup("padding-left", "padding", &zero);
            let padding_right = style.lookup("padding-right", "padding", &zero);
    
            let d = &mut self.dimensions;
    
            d.padding.left = padding_left.to_px();
            d.padding.right = padding_right.to_px();
    
            d.border.left = border_left.to_px();
            d.border.right = border_right.to_px();
    
            d.margin.left = margin_left.to_px();
            d.margin.right = margin_right.to_px();
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
            self.dimensions.inner.left = self.dimensions.margin.left + self.dimensions.border.left + self.dimensions.padding.left;
            self.dimensions.inner.top = self.dimensions.margin.top + self.dimensions.border.top + self.dimensions.padding.top;
        }
        

        // 2. recursive -> vertical(block, anonymous block)
        let mut children_sum_height = 0f32;
        let mut children_max_width = 0f32;
        for child in &mut self.children {
            child.layout(self_as_container_width.clone(), self_as_context_constraints_width.clone());
            child.dimensions.box_offset.top = children_sum_height;
            children_sum_height += child.dimensions.margin_box().height;
            children_max_width = children_max_width.max(child.dimensions.margin_box().width);
        }

        // 3. self main: height -> (specific, auto by children sum)
        let mut height = style.value("height").unwrap_or(Value::Keyword("auto".to_string()));
        if height.is_specific_length() {
            self.dimensions.inner.height = height.to_px();
        }
        else if height.is_auto() {
            self.dimensions.inner.height = children_sum_height;
        }

        // 4. self cross: width -> (specific, max by children)
        if is_self_no_filled_auto {
            self.dimensions.inner.width = children_max_width;
            self.dimensions.inner.width = children_max_width;
        }

        // 5. fill children: width -> block (empty auto -> fill one line)
        if is_self_no_filled_auto {
            let underflow_content = self.dimensions.inner.width - {
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
                if !matches!(child.box_type, BoxType::AnonymousBlock) && child.is_width_auto() {
                    // auto and not anonymous -> retake one line
                    child.layout(self_as_container_width.clone(), self_as_context_constraints_width.clone());
                }
            }
        }

        // 6. fixing children #[cfg(not(margin-auto))]
        for child in &mut self.children {
            // edge size auto by position block
            // self.dimensions.box_offset.left; // no margin so to do nothing
        }


    }

    pub fn calc_abs(&mut self) {
        self.dimensions.box_abs.x = self.dimensions.get_left();
        self.dimensions.box_abs.y = self.dimensions.get_top();
        for child in &mut self.children {
            child.traversal_calc_abs(self.dimensions.box_abs);
        }
    }

    pub fn traversal_calc_abs(&mut self, parent_abs: BoxAbs) {
        self.dimensions.box_abs.x = parent_abs.x + self.dimensions.get_left();
        self.dimensions.box_abs.y = parent_abs.y + self.dimensions.get_top();
        for child in &mut self.children {
            child.traversal_calc_abs(self.dimensions.box_abs);
        }
    }

    // Where a new inline child should go.
    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => self,
            BoxType::BlockNode(_) | BoxType::InlineBlockNode(_) => {
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

        layout_tree.layout(Value::Length(200.0, Unit::Px), Value::Length(200.0, Unit::Px));

        println!("{:#?}", layout_tree);

        assert_eq!(layout_tree.dimensions.margin_box().width, 200.0);
        assert_eq!(layout_tree.dimensions.padding_box().width, 160.0);
        assert_eq!(layout_tree.dimensions.inner.width, 140.0);

        assert_eq!(layout_tree.children[0].dimensions.margin_box().width, 140.0);
        assert_eq!(
            layout_tree.children[0].dimensions.padding_box().width,
            100.0
        );
        assert_eq!(layout_tree.children[0].dimensions.inner.width, 80.0);

        assert_eq!(layout_tree.children[0].dimensions.margin_box().height, 60.0);
        assert_eq!(layout_tree.dimensions.margin_box().height, 120.0);
    }
}
