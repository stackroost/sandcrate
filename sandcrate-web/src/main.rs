use dioxus::prelude::*;

fn main() {
    launch(app);
}

fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        button { onclick: |_| {
            gloo::console::log!("TODO: trigger plugin");
        }, "Run Plugin" }
    })
}
