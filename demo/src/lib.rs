use js_sys::Date;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let textarea = document.get_element_by_id("source").unwrap();

    let mut parser = markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::extra::add(&mut parser);

    let mut last_exec = 0f64;
    let timeout_ = std::rc::Rc::new(std::cell::Cell::new(None));

    let timeout = timeout_.clone();
    let do_render = Closure::<dyn FnMut()>::new(move || {
        let src = textarea.dyn_ref::<web_sys::HtmlTextAreaElement>().unwrap().value();
        let ast =  parser.parse(&src);
        let html = ast.render();
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let preview = document.get_element_by_id("result-preview").expect("no #result-preview");
        preview.set_inner_html(&html);
        let result_html = document.get_element_by_id("result-html").expect("no #result-html");
        result_html.set_text_content(Some(&html));

        let mut ast_str = String::new();
        ast.walk(|node, depth| {
            ast_str.push_str(&"    ".repeat(depth as usize));
            let name = &node.name()[node.name().rfind("::").map(|x| x+2).unwrap_or_default()..];
            if let Some(data) = node.cast::<markdown_it::parser::inline::Text>() {
                ast_str.push_str(&format!("{}: {:?}\n", name, data.content));
            } else if let Some(data) = node.cast::<markdown_it::parser::inline::TextSpecial>() {
                ast_str.push_str(&format!("{}: {:?}\n", name, data.content));
            } else {
                ast_str.push_str(&format!("{}\n", name));
            }
        });
        let result_ast = document.get_element_by_id("result-ast").expect("no #result-ast");
        result_ast.set_text_content(Some(&ast_str));
        timeout.set(None);
    });

    let timeout = timeout_.clone();
    timeout.set(window.set_timeout_with_callback(do_render.as_ref().unchecked_ref()).ok());

    let timeout = timeout_;
    const DEBOUNCE_TIMEOUT : i32 = 100; // ms
    let input_handler = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
        let now = Date::now();
        if (now - last_exec).abs() < DEBOUNCE_TIMEOUT as f64 {
            if timeout.get().is_none() {
                timeout.set(
                    window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        do_render.as_ref().unchecked_ref(),
                        DEBOUNCE_TIMEOUT
                    ).ok()
                );
            }
            return;
        }

        last_exec = now;
        timeout.set(window.set_timeout_with_callback(do_render.as_ref().unchecked_ref()).ok());
    });

    let textarea = document.get_element_by_id("source").unwrap();
    textarea.add_event_listener_with_callback("input", input_handler.as_ref().unchecked_ref()).unwrap();
    input_handler.forget();

    let tab_click = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let tabs = document.query_selector_all("#tabs-toggle a.nav-link.active").unwrap();
        for idx in 0..tabs.length() {
            let tab = tabs.get(idx).unwrap();
            let el = tab.dyn_into::<web_sys::HtmlElement>().unwrap();
            el.class_list().remove_1("active").unwrap();
        }
        let el = event.target().unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
        el.class_list().add_1("active").unwrap();

        let contents = document.query_selector_all(".tab-content").unwrap();
        for idx in 0..contents.length() {
            let el = contents.get(idx).unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
            el.style().set_property("display", "none").unwrap();
        }

        let toggle = el.dataset().get("toggle").expect("no data-toggle attribute on anchor");
        let el = document.get_element_by_id(&toggle).unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
        el.style().set_property("display", "block").unwrap();
        event.prevent_default();
    });

    let tabs = document.query_selector_all("#tabs-toggle a.nav-link").unwrap();
    for idx in 0..tabs.length() {
        tabs.get(idx).unwrap().add_event_listener_with_callback("click", tab_click.as_ref().unchecked_ref()).unwrap();
    }
    tabs.get(0).unwrap().dispatch_event(&web_sys::Event::new("click").unwrap()).unwrap();
    tab_click.forget();
}
