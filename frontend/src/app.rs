use anyhow::Error;
use serde_derive::{Deserialize, Serialize};
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use common::{Connection};
use bincode::{serialize};


pub enum WsAction {
    Connect,
    Disconnect,
    Lost,
    Join,
}

pub struct App {
    link: ComponentLink<Self>,
    fetching: bool,
    ft: Option<FetchTask>,
    ws: Option<WebSocketTask>,
    login: Option<String>,
    channel: Option<String>,
    data: Option<u32>,
}

pub enum Msg {
    NoOp,
    //    FetchData,
    //    FetchReady(Result<DataFromFile, Error>),
    Ignore,
    WsAction(WsAction),
    WsReady(Result<WsResponse, Error>),
    UpdateLogin(String),
    UpdateChannel(String),
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
            login: None,
            channel: None,
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
            Msg::UpdateLogin(login) => self.login = Some(login),
            Msg::UpdateChannel(channel) => self.channel = Some(channel),
            Msg::WsAction(action) => match action {
                WsAction::Connect => {
                    let callback = self.link.callback(|Json(data)| Msg::WsReady(data));
                    let notification = self.link.callback(|status| match status {
                        WebSocketStatus::Opened => Msg::WsAction(WsAction::Join),
                        WebSocketStatus::Closed | WebSocketStatus::Error => WsAction::Lost.into(),
                    });
                    let task =
                        WebSocketService::connect("ws://localhost:9001/", callback, notification)
                            .unwrap();
                    self.ws = Some(task);
                },
                WsAction::Disconnect => {
                    self.ws.take();
                }
                WsAction::Lost => {
                    self.ws = None;
                },
                WsAction::Join => {
                    let msg = bincode::serialize(&common::Message::Connection(common::Connection{time:chrono::Utc::now(), login:self.login.as_ref().unwrap().clone(), channel: self.channel.as_ref().unwrap().clone()})).unwrap();
                    self.ws.as_mut().unwrap().send_binary(Ok(msg));
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
                <div>
                <label>{"Login : "}<input type="text" disabled=self.ws.is_some() oninput=self.link.callback(|x: InputData| Msg::UpdateLogin(x.value))/></label>
                <label>{"Channel : "}<input type="text" disabled=self.ws.is_some() oninput=self.link.callback(|x: InputData| Msg::UpdateChannel(x.value))/></label>
                <button disabled=self.ws.is_some() || (self.login.is_none() || self.channel.is_none())
                onclick=self.link.callback(|_| WsAction::Connect)>
                { "Connect To WebSocket" }
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
