pub mod renderer;
pub mod parser;
pub mod syntax;

use crate::parser::internals::sourcemap::SourcePos;
use downcast_rs::{Downcast, impl_downcast};
use parser::internals::common::escape_html;
use std::fmt::Debug;
use std::rc::Rc;

/// Each node outputs its HTML using this API.
///
/// Renderer is a struct that walks through AST and collects HTML from each node
/// into internal buffer. You can implement your own renderer if you want to add
/// custom HTML attributes or change whitespacing. See
/// [DefaultRenderer](renderer::DefaultRenderer) for an example implementation.
pub trait Renderer {
    /// Write opening html tag with attributes, e.g. `<a href="url">`.
    fn open(&mut self, tag: &str, attrs: &[(&str, &str)]);
    /// Write closing html tag, e.g. `</a>`.
    fn close(&mut self, tag: &str);
    /// Write self-closing html tag with attributes, e.g. `<img src="url"/>`.
    fn self_close(&mut self, tag: &str, attrs: &[(&str, &str)]);
    /// Loop through child nodes and render each one.
    fn contents(&mut self, nodes: &[Node]);
    /// Write line break (`\n`). Default renderer ignores it if last char in the buffer is `\n` already.
    fn cr(&mut self);
    /// Write plain text with escaping, `<div>` -> `&lt;div&gt;`.
    fn text(&mut self, text: &str);
    /// Write plain text without escaping, `<div>` -> `<div>`.
    fn text_raw(&mut self, text: &str);
}

/// Single node in the CommonMark AST.
#[derive(Debug)]
pub struct Node {
    /// Array of child nodes.
    pub children: Vec<Node>,

    /// Source mapping info.
    pub srcmap: Option<SourcePos>,

    /*pub render: enum {
        NormalTag(name),
        SelfClosingTag(name),
        Text(content),
        TextRaw(content),
        Custom(Box<dyn Fn() -> String>),
        None,
    }

    pub attrs: ...,

    pub spacing: ...,*/

    /// Type name, used for debugging.
    name: &'static str,

    /// Storage for arbitrary token-specific data.
    value: Box<dyn NodeValue>,
}

impl Node {
    /// Create a new [Node](Node) with a custom value.
    pub fn new<T: NodeValue>(value: T) -> Self {
        Self {
            children:  Vec::new(),
            srcmap:    None,
            name:      std::any::type_name::<T>(),
            value:     Box::new(value),
        }
    }

    /// Return std::any::type_name() of node value.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Check that this node value is of given type.
    pub fn is<T: NodeValue>(&self) -> bool {
        self.value.is::<T>()
    }

    /// Downcast node value to specific type.
    pub fn cast<T: NodeValue>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }

    /// Downcast node value to specific type.
    pub fn cast_mut<T: NodeValue>(&mut self) -> Option<&mut T> {
        self.value.downcast_mut::<T>()
    }

    /// Render this node to html using Renderer API.
    pub fn render(&self, fmt: &mut dyn Renderer) {
        self.value.render(self, fmt);
    }

    /// Render this node to html using Renderer API.
    //pub fn render2(&self) -> String {
    //    self.value.render2(self).render(self)
    //}

    /// Replace custom value with another value (this is roughly equivalent
    /// to replacing the entire node and copying children and sourcemaps).
    pub fn replace<T: NodeValue>(&mut self, value: T) {
        self.name = std::any::type_name::<T>();
        self.value = Box::new(value);
    }

    /// Execute function `f` recursively on every member of AST tree
    /// (using preorder deep-first search).
    pub fn walk(&self, mut f: impl FnMut(&Node, u32)) {
        let mut stack = vec![(self, 0)];

        while let Some((node, depth)) = stack.pop() {
            f(node, depth);
            for n in node.children.iter().rev() {
                stack.push((n, depth + 1));
            }
        }
    }

    /// Execute function `f` recursively on every member of AST tree
    /// (using preorder deep-first search).
    pub fn walk_mut(&mut self, mut f: impl FnMut(&mut Node, u32)) {
        let mut stack = vec![(self, 0)];

        while let Some((node, depth)) = stack.pop() {
            f(node, depth);
            for n in node.children.iter_mut().rev() {
                stack.push((n, depth + 1));
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HtmlSpacing {
    None        = 0,
    Before      = 1,
    After       = 2,
    BeforeInner = 4,
    AfterInner  = 8,
    Around      = 1 + 2,
    All         = 1 + 2 + 4 + 8,
}

#[derive(Debug)]
pub struct HtmlElement {
    pub tag: &'static str,
    pub attrs: Vec<(&'static str, String)>,
    pub children: Option<Vec<Html>>,
    pub spacing: HtmlSpacing,
}

/*impl HtmlElement {
    fn render(self, node: &Node) -> String {
        let mut attrs = String::new();

        for (name, value) in self.attrs {
            attrs.push(' ');
            attrs.push_str(&escape_html(name));
            attrs.push('=');
            attrs.push('"');
            attrs.push_str(&escape_html(&value));
            attrs.push('"');
        }

        format!("<{}{}>{}</{}>", self.tag, attrs, self.children.into_iter().map(|x| x.render(node)).collect::<String>(), self.tag)
    }
}*/

#[derive(Debug)]
pub enum Html {
    Element(HtmlElement),
    Text(String),
    RawText(String),
    Children,
    NodeList(Vec<Html>),
}

/*impl Html {
    pub fn render(self, node: &Node) -> String {
        match self {
            Html::Text(x) => escape_html(&x).to_string(),
            Html::RawText(x) => x,
            Html::Children => node.children.iter().map(|x| x.render2()).collect::<String>(),
            Html::Element(el) => el.render(node),
            Html::NodeList(list) => list.into_iter().map(|x| x.render(node)).collect::<String>(),
        }
    }
}*/

/// Contents of the specific AST node.
pub trait NodeValue : Debug + Downcast {
    /// Output HTML corresponding to this node using Renderer API.
    ///
    /// Example implementation looks like this:
    /// ```rust
    /// # const IGNORE : &str = stringify! {
    /// fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
    ///    fmt.open("div", &[]);
    ///    fmt.contents(&node.children);
    ///    fmt.close("div");
    ///    fmt.cr();
    /// }
    /// # };
    /// ```
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let _ = fmt;
        unimplemented!("{} doesn't implement render", node.name());
    }

    fn render2(&self, node: &Node) -> Html {
        Html::Children
    }
}

impl_downcast!(NodeValue);

pub fn render2(root: Node) -> String {
    enum Operation {
        EnterTag((Html, Rc<Vec<Node>>)),
        ExitTag((&'static str, HtmlSpacing)),
    }

    let mut result = String::new();
    let mut stack = vec![
        Operation::EnterTag((root.value.render2(&root), Rc::new(root.children)))
    ];

    loop {
        match stack.pop() {
            Some(Operation::EnterTag((html, children))) => {
                match html {
                    Html::Text(text) => result += &escape_html(&text),
                    Html::RawText(text) => result += &text,
                    Html::Children => {
                        let children = Rc::try_unwrap(children).unwrap_or_default();
                        for child in children.into_iter().rev() {
                            stack.push(Operation::EnterTag((child.value.render2(&child), Rc::new(child.children))));
                        }
                    }
                    Html::Element(el) => {
                        if el.spacing as u8 & HtmlSpacing::Before as u8 != 0 {
                            match result.as_bytes().last() {
                                Some(b'\n') | None => {}
                                Some(_) => result.push('\n')
                            }
                        }
                        result.push('<');
                        result.push_str(el.tag);
                        for (name, value) in el.attrs {
                            result.push(' ');
                            result.push_str(&escape_html(name));
                            result.push('=');
                            result.push('"');
                            result.push_str(&escape_html(&value));
                            result.push('"');
                        }
                        if el.children.is_none() {
                            result.push(' ');
                            result.push('/');
                            result.push('>');
                            if el.spacing as u8 & HtmlSpacing::After as u8 != 0 {
                                match result.as_bytes().last() {
                                    Some(b'\n') | None => {}
                                    Some(_) => result.push('\n')
                                }
                            }
                        } else {
                            result.push('>');
                            if el.spacing as u8 & HtmlSpacing::BeforeInner as u8 != 0 {
                                match result.as_bytes().last() {
                                    Some(b'\n') | None => {}
                                    Some(_) => result.push('\n')
                                }
                            }

                            stack.push(Operation::ExitTag((el.tag, el.spacing)));

                            for html in el.children.unwrap().into_iter().rev() {
                                stack.push(Operation::EnterTag((html, Rc::clone(&children))));
                            }
                        }
                    }
                    Html::NodeList(list) => {
                        for html in list.into_iter().rev() {
                            stack.push(Operation::EnterTag((html, Rc::clone(&children))))
                        }
                    }
                }
            }
            Some(Operation::ExitTag((tag, spacing))) => {
                if spacing as u8 & HtmlSpacing::AfterInner as u8 != 0 {
                    match result.as_bytes().last() {
                        Some(b'\n') | None => {}
                        Some(_) => result.push('\n')
                    }
                }
                result.push('<');
                result.push('/');
                result.push_str(tag);
                result.push('>');
                if spacing as u8 & HtmlSpacing::After as u8 != 0 {
                    match result.as_bytes().last() {
                        Some(b'\n') | None => {}
                        Some(_) => result.push('\n')
                    }
                }
            }
            None => break,
        }
    }

    result

    /*//node.render2()
    let mut result = String::new();
    let mut stack_enter = vec![ root ];
    //let mut stack_exit = vec![];

    while let Some(node) = stack_enter.pop() {
        let html = node.value.render2(&node);

        match html {
            Html::Text(text) => result += &escape_html(&text),
            Html::RawText(text) => result += &text,
            Html::Children => {
                //node.children.iter().map(|x| x.render2()).collect::<String>(),
                for child in node.children.into_iter().rev() {
                    stack_enter.push(child);
                }
            }
            //Html::Element(el) => el.render(node),
            //Html::NodeList(list) => list.into_iter().map(|x| x.render(node)).collect::<String>(),
            _=>{}
        }

        //stack_exit.push(node);
*/
        /*for child in node.children {
            stack_enter.push(child);
        }*/
   //// }

/*
    let html = node.value.render2(&node);

    match html {
        Html::Text(text) => result += &escape_html(&text),
        Html::RawText(text) => result += &text,
        //Html::Children => node.children.iter().map(|x| x.render2()).collect::<String>(),
        //Html::Element(el) => el.render(node),
        //Html::NodeList(list) => list.into_iter().map(|x| x.render(node)).collect::<String>(),
        _=>{}
    }
*/
   // return result;
}
