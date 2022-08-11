use js_sys::Date;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let textarea = document.get_element_by_id("source").unwrap();
    let output = document.get_element_by_id("target").unwrap();

    let mut parser = markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::extra::add(&mut parser);

    let mut last_exec = 0f64;
    let timeout_ = std::rc::Rc::new(std::cell::Cell::new(None));

    let timeout = timeout_.clone();
    let do_render = Closure::<dyn FnMut()>::new(move || {
        let src = textarea.dyn_ref::<web_sys::HtmlTextAreaElement>().unwrap().value();
        let html = parser.parse(&src).render();
        output.set_inner_html(&html);
        timeout.set(None);
    });

    let timeout = timeout_.clone();
    timeout.set(window.set_timeout_with_callback(do_render.as_ref().unchecked_ref()).ok());

    let timeout = timeout_;
    const DEBOUNCE_TIMEOUT : i32 = 100; // ms
    let input_handler = Closure::<dyn FnMut(_)>::new(move |_: web_sys::InputEvent| {
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
}
