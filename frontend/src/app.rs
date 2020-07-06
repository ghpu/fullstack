use anyhow::Error;
use common::{Annotation,Case, IntentMapping};
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

pub struct App {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    console_service: ConsoleService,
    fetching: bool,
    data: Option<Case>,
    ft: Option<FetchTask>,
}

pub enum Msg {
    NoOp,
    FetchData,
    FetchReady(Result<Case, Error>),
    Ignore,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            console_service: ConsoleService::new(),
            fetch_service: FetchService::new(),
            link,
            fetching: false,
            data: None,
            ft: None,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link.callback(|_| Msg::FetchData).emit("");
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => {}
            Msg::FetchData => {
                self.fetching = true;
                let callback = self.link.callback(
                    move |response: Response<Json<Result<Case, Error>>>| {
                        let (meta, Json(data)) = response.into_parts();
                        if meta.status.is_success() {
                            Msg::FetchReady(data)
                        } else {
                            Msg::Ignore
                        }
                    },
                );
                let request = Request::get("/data").body(Nothing).unwrap();
                let task = self.fetch_service.fetch(request, callback).unwrap();
                self.ft = Some(task);
            }
            Msg::FetchReady(response) => {
                self.fetching = false;
                self.data = Some(response.unwrap());
            }
            Msg::Ignore => {
                self.fetching = false;
                self.data = None;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div><h1>{ "Hello world!, " }</h1><p>{"Loading in progress: "}{self.fetching}</p><p>{ format!("{:?}", self.data) }</p></div>
        }
    }

    fn change(&mut self, _: <Self as yew::html::Component>::Properties) -> bool {
        false
    }
}
