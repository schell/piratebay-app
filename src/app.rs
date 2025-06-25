use futures_lite::FutureExt;
use human_repr::HumanCount;
use mogwai::{future::MogwaiFutureExt, web::prelude::*};
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
struct TorrentView<V: View> {
    #[child]
    wrapper: V::Element,
}

pub fn format_unix_timestamp_with_locale(seconds: i64) -> String {
    // Convert seconds to milliseconds
    let milliseconds = seconds as f64 * 1000.0;
    // Create a new Date object
    let date = web_sys::js_sys::Date::new(&milliseconds.into());
    // Get the user's locale
    let user_locale =
        web_sys::js_sys::Reflect::get(&web_sys::js_sys::global(), &"navigator".into())
            .and_then(|navigator| web_sys::js_sys::Reflect::get(&navigator, &"language".into()))
            .unwrap_or_else(|_| JsValue::from_str("en-US"))
            .as_string()
            .unwrap_or_else(|| "en-US".to_string());
    // Format the date using the user's locale
    date.to_locale_string(&user_locale, &JsValue::undefined())
        .into()
}

impl<V: View> TorrentView<V> {
    fn new(torrent: &Torrent) -> Self {
        let added = if V::is_view::<Web>() {
            format_unix_timestamp_with_locale(torrent.added_i64())
        } else {
            torrent.added.clone()
        };
        rsx! {
            let wrapper = tr() {
                td(class = "torrent-name") { {&torrent.name} }
                td() { {&added} }
                td() { {&torrent.seeders} }
                td() { {&torrent.leechers} }
                td() { {format!("{}", torrent.size_bytes().human_count_bytes())} }
                td(class = "torrent-username") { {&torrent.username} }
            }
        }
        Self { wrapper }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SortColumn {
    Name,
    Date,
    Seeders,
    Leechers,
    Size,
    Uploader,
}

impl SortColumn {
    fn header_view<V: View>(&self, current_sorting: &Sort) -> V::Element {
        let name = match self {
            SortColumn::Name => "Name",
            SortColumn::Date => "Date Added",
            SortColumn::Seeders => "Seeders",
            SortColumn::Leechers => "Leechers",
            SortColumn::Size => "Size",
            SortColumn::Uploader => "Uploader",
        };
        let is_selected = Some(self) == current_sorting.column.as_ref();
        let dir = is_selected.then_some(
            match current_sorting.direction {
                Direction::Descending => "🔽",
                Direction::Ascending => "🔼",
            }
            .into_text::<V>(),
        );
        rsx! {
            let wrapper = span() {
                {name.into_text::<V>()}
                span(class = "direction") {{dir}}
            }
        }
        wrapper
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Direction {
    #[default]
    Descending,
    Ascending,
}

#[derive(Clone, Default, PartialEq)]
struct Sort {
    column: Option<SortColumn>,
    direction: Direction,
}

#[derive(ViewChild)]
struct SearchResults<V: View> {
    #[child]
    wrapper: V::Element,
    torrents: Proxy<Vec<Torrent>>,
    sort: Proxy<Sort>,
    on_click_name: V::EventListener,
    on_click_date: V::EventListener,
    on_click_seeders: V::EventListener,
    on_click_leechers: V::EventListener,
    on_click_size: V::EventListener,
    on_click_uploader: V::EventListener,
}

impl<V: View> Default for SearchResults<V> {
    fn default() -> Self {
        use SortColumn::*;
        let mut torrents = Proxy::<Vec<Torrent>>::default();
        let mut sort = Proxy::<Sort>::default();
        rsx! {
            let wrapper = div(class = "row search-results", style:display = "none") {
                fieldset() {
                    legend() { "Results:" }
                    table() {
                        tr() {
                            th(on:click = on_click_name) {{sort(s => Name.header_view::<V>(s))}}
                            th(on:click = on_click_date) {{sort(s => Date.header_view::<V>(s))}}
                            th(on:click = on_click_seeders) {{sort(s => Seeders.header_view::<V>(s) )}}
                            th(on:click = on_click_leechers) {{sort(s => Leechers.header_view::<V>(s) )}}
                            th(on:click = on_click_size) {{sort(s => Size.header_view::<V>(s) )}}
                            th(on:click = on_click_uploader) {{sort(s => Uploader.header_view::<V>(s))}}
                        }
                        {torrents(ts => ts.iter().map(TorrentView::<V>::new).collect::<Vec<_>>())}
                    }
                }
            }
        }

        Self {
            wrapper,
            torrents,
            on_click_name,
            on_click_date,
            on_click_seeders,
            on_click_leechers,
            on_click_size,
            on_click_uploader,
            sort,
        }
    }
}

impl<V: View> SearchResults<V> {
    async fn step(&mut self) {
        use SortColumn::*;
        let events = vec![
            self.on_click_name.next().map(|_| Name).boxed_local(),
            self.on_click_date.next().map(|_| Date).boxed_local(),
            self.on_click_seeders.next().map(|_| Seeders).boxed_local(),
            self.on_click_leechers
                .next()
                .map(|_| Leechers)
                .boxed_local(),
            self.on_click_size.next().map(|_| Size).boxed_local(),
            self.on_click_uploader
                .next()
                .map(|_| Uploader)
                .boxed_local(),
        ];
        let current_sort = self.sort.as_ref().clone();
        let column = mogwai::future::race_all(events).await;
        let direction = if Some(column) == current_sort.column {
            if current_sort.direction == Direction::Descending {
                Direction::Ascending
            } else {
                Direction::Descending
            }
        } else {
            current_sort.direction
        };
        let sort = Sort {
            column: Some(column),
            direction,
        };
        if sort != current_sort {
            self.torrents.modify(|ts| {
                ts.sort_by(|a, b| {
                    let ord = match column {
                        Name => a.name.cmp(&b.name),
                        Date => a.added_i64().cmp(&b.added_i64()),
                        Seeders => a.seeders_i64().cmp(&b.seeders_i64()),
                        Leechers => a.leechers_i64().cmp(&b.leechers_i64()),
                        Size => a.size_bytes().cmp(&b.size_bytes()),
                        Uploader => a.username.cmp(&b.username),
                    };
                    if direction == Direction::Descending {
                        ord.reverse()
                    } else {
                        ord
                    }
                });
            });
        }
        self.sort.set(sort);
    }
}

#[derive(ViewChild)]
pub struct App<V: View> {
    #[child]
    wrapper: V::Element,
    input: V::Element,
    on_submit_query: V::EventListener,
    status_text: V::Text,
    search_results: SearchResults<V>,
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
                    button(type="submit"){ "Search" }
                }
                p() {
                    let status_text = ""
                }
                let search_results = {SearchResults::default()}
            }
        }
        Self {
            wrapper,
            input,
            on_submit_query,
            status_text,
            search_results,
        }
    }
}

enum Step<V: View> {
    Results,
    Submit(V::Event),
}

impl<V: View> App<V> {
    pub async fn step(&mut self) {
        log::info!("step");

        let submission = self.on_submit_query.next().map(Step::Submit);
        let sorting = self.search_results.step().map(|_| Step::Results);
        let ev: Step<V> = submission.or(sorting).await;
        match ev {
            Step::Results => {}
            Step::Submit(ev) => {
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
                        self.search_results.torrents.set(torrents);
                        self.search_results.wrapper.set_style("display", "block");
                    }
                    Err(Error { msg }) => {
                        self.status_text.set_text(msg);
                    }
                }
            }
        }
    }
}
