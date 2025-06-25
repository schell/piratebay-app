use mogwai::web::prelude::*;
use pb_wire_types::*;
use wasm_bindgen::prelude::*;

mod invoke {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
        async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    }

    fn deserialize_as<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T, Error> {
        match serde_wasm_bindgen::from_value::<T>(value) {
            Ok(t) => Ok(t),
            Err(e) => {
                log::error!("e: {e:#?}");
                Err(Error {
                    msg: "Could not deserialize".into(),
                })
            }
        }
    }

    pub async fn cmd<T: serde::Serialize, X: serde::de::DeserializeOwned>(
        name: &str,
        args: &T,
    ) -> Result<X, Error> {
        let value = serde_wasm_bindgen::to_value(args)
            .map_err(|e| format!("could not serialize {}: {e}", std::any::type_name::<T>()))?;
        let result = invoke(name, value).await;
        log::info!("result: {result:#?}");
        match result {
            Ok(value) => deserialize_as::<X>(value),
            Err(e) => Err(deserialize_as::<Error>(e)?),
        }
    }
}

pub async fn search(query: &str) -> Result<Vec<Torrent>, Error> {
    #[derive(serde::Serialize)]
    struct Query<'a> {
        query: &'a str,
    }

    invoke::cmd("search", &Query { query }).await
}

#[derive(ViewChild)]
pub struct App<V: View> {
    #[child]
    wrapper: V::Element,
    input: V::Element,
    on_submit_query: V::EventListener,
    status_text: V::Text,
}

impl<V: View> Default for App<V> {
    fn default() -> Self {
        rsx! {
            let wrapper = main(class="container") {
                h1() { "Welcome to piratebay-app" }
                p() { "Enter a search query" }
                form(class="row", on:submit = on_submit_query) {
                    let input = input(
                        id="greet-input",
                        placeholder="Enter a name...",
                    ){}
                    button(type="submit"){ "Greet" }
                }
                p() {
                    let status_text = ""
                }
            }
        }
        Self {
            wrapper,
            input,
            on_submit_query,
            status_text,
        }
    }
}

impl<V: View> App<V> {
    pub async fn step(&mut self) {
        log::info!("step");
        let ev = self.on_submit_query.next().await;
        log::info!("submit");
        ev.dyn_ev(|ev: &web_sys::Event| ev.prevent_default());
        let search_query = self
            .input
            .dyn_el(|input: &web_sys::HtmlInputElement| input.value())
            .unwrap_or_default();
        self.status_text
            .set_text(format!("Searching for '{search_query}'..."));

        match search(&search_query).await {
            Ok(torrents) => {
                self.status_text
                    .set_text(format!("Found {} results.", torrents.len()));
            }
            Err(Error { msg }) => {
                self.status_text.set_text(msg);
            }
        }
    }
}
