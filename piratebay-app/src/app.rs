use futures_lite::FutureExt;
// use leptos::task::spawn_local;
// use leptos::{ev::SubmitEvent, prelude::*};
use mogwai::{future::MogwaiFutureExt, web::prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GreetArgs {
    name: String,
}

#[derive(ViewChild)]
pub struct App<V: View> {
    #[child]
    wrapper: V::Element,
    input: V::Element,
    on_submit_greeting: V::EventListener,
    greeting_text: V::Text,
}

impl<V: View> Default for App<V> {
    fn default() -> Self {
        rsx! {
            let wrapper = main(class="container") {
                h1() { "Welcome to Tauri + Mogwai"}

                div(class="row") {
                    a(href="https://tauri.app", target="_blank") {
                        img(src="public/tauri.svg", class="logo tauri", alt="Tauri logo"){}
                    }
                    a(href="https://docs.rs/mogwai/", target="_blank") {
                        img(src="public/mogwai.svg", class="logo mogwai", alt="Mogwai logo"){}
                    }
                }
                p() { "Click on the Tauri and Mogwai logos to learn more." }

                form(class="row", on:submit = on_submit_greeting) {
                    let input = input(
                        id="greet-input",
                        placeholder="Enter a name...",
                    ){}
                    button(type="submit"){ "Greet" }
                }
                p() {
                    let greeting_text = ""
                }
            }
        }
        Self {
            wrapper,
            input,
            on_submit_greeting,
            greeting_text,
        }
    }
}

impl<V: View> App<V> {
    pub async fn step(&mut self) {
        log::info!("step");
        let ev = self.on_submit_greeting.next().await;
        log::info!("submit");
        ev.dyn_ev(|ev: &web_sys::Event| ev.prevent_default());
        let name = self
            .input
            .dyn_el(|input: &web_sys::HtmlInputElement| input.value())
            .unwrap_or_default();
        let args = serde_wasm_bindgen::to_value(&GreetArgs { name }).unwrap();
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        let new_msg = invoke("greet", args).await.as_string().unwrap();
        self.greeting_text.set_text(new_msg);
    }
}
