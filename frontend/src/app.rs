use anyhow::Error;
use serde_derive::{Deserialize, Serialize};
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

type AsBinary = bool;

pub enum WsAction {
    Connect,
    SendData(AsBinary),
    Disconnect,
    Lost,
}

pub struct App {
    link: ComponentLink<Self>,
    fetching: bool,
    data: Option<u32>,
    ft: Option<FetchTask>,
    ws: Option<WebSocketTask>,
}

pub enum Msg {
    NoOp,
    //    FetchData,
    //    FetchReady(Result<DataFromFile, Error>),
    Ignore,
    WsAction(WsAction),
    WsReady(Result<WsResponse, Error>),
}

impl From<WsAction> for Msg {
    fn from(action: WsAction) -> Self {
        Msg::WsAction(action)
    }
}

#[derive(Deserialize, Debug)]
pub struct WsResponse {
    value: u32,
}

#[derive(Serialize, Debug)]
pub struct WsRequest {
    value: u32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
            fetching: false,
            data: None,
            ft: None,
            ws: None,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            //self.link.callback(|_| Msg::FetchData).emit("");
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => {}
            Msg::WsAction(action) => match action {
                WsAction::Connect => {
                    let callback = self.link.callback(|Json(data)| Msg::WsReady(data));
                    let notification = self.link.callback(|status| match status {
                        WebSocketStatus::Opened => Msg::Ignore,
                        WebSocketStatus::Closed | WebSocketStatus::Error => WsAction::Lost.into(),
                    });
                    let task =
                        WebSocketService::connect("ws://localhost:9001/", callback, notification)
                            .unwrap();
                    self.ws = Some(task);
                }
                WsAction::SendData(binary) => {
                    let request = WsRequest { value: 321 };
                    if binary {
                        self.ws.as_mut().unwrap().send_binary(Json(&request));
                    } else {
                        self.ws.as_mut().unwrap().send(Json(&request));
                    }
                }
                WsAction::Disconnect => {
                    self.ws.take();
                }
                WsAction::Lost => {
                    self.ws = None;
                }
            },
            Msg::WsReady(response) => {
                self.data = response.map(|data| data.value).ok();
            }
            /*Msg::FetchData => {
            self.fetching = true;
            let callback = self.link.callback(
            move |response: Response<Json<Result<DataFromFile, Error>>>| {
            let (meta, Json(data)) = response.into_parts();
            if meta.status.is_success() {
            Msg::FetchReady(data)
            } else {
            Msg::Ignore
            }
            },
            );
            let request = Request::get("/data.json").body(Nothing).unwrap();
            let task = self.fetch_service.fetch(request, callback).unwrap();
            self.ft = Some(task);
            }
            Msg::FetchReady(response) => {
            self.fetching = false;
            self.data = response.map(|data| data.name).ok();
            }*/
            Msg::Ignore => {
                self.fetching = false;
                self.data = None;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <div>{ "Hello world!, " }{self.fetching}{ format!("{:?}", self.data) }</div>
                <div><button disabled=self.ws.is_some()
                onclick=self.link.callback(|_| WsAction::Connect)>
                { "Connect To WebSocket" }
            </button>
                <button disabled=self.ws.is_none()
                onclick=self.link.callback(|_| WsAction::SendData(false))>
                { "Send To WebSocket" }
            </button>
                <button disabled=self.ws.is_none()
                onclick=self.link.callback(|_| WsAction::SendData(true))>
                { "Send To WebSocket [binary]" }
            </button>
                <button disabled=self.ws.is_none()
                onclick=self.link.callback(|_| WsAction::Disconnect)>
                { "Close WebSocket connection" }
            </button>
                </div>
                </>
        }
    }

    fn change(&mut self, _: <Self as yew::html::Component>::Properties) -> bool {
        false
    }
}
