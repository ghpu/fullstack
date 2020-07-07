use anyhow::Error;
use common::{Annotation,Case, Corpus,IntentMapping};
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use std::hash::{Hash,Hasher};


pub struct App {
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    console_service: ConsoleService,
    fetching: bool,
    data: Option<Corpus>,
    ft: Option<FetchTask>,
}

pub enum Msg {
    NoOp,
    FetchData,
    FetchReady(Result<Corpus, Error>),
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
                    move |response: Response<Json<Result<Corpus, Error>>>| {
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
            <div><h1>{ "Hello world! "}</h1><p>{"Loading in progress: "}{self.fetching}</p><p>{ if let Some(d) = &self.data { display_corpus(&d)} else {html!{}}}</p></div>
        }
    }

    fn change(&mut self, _: <Self as yew::html::Component>::Properties) -> bool {
        false
    }
}


fn display_corpus(corpus: &common::Corpus) -> Html {
    html! {
        <table style="border-collapse:collapse;">
            <thead>
            <tr style="background-color:lightgrey;"><th colspan="6">{"Corpus details"}</th></tr>
            <tr style="background-color:lightgrey;"><th>{"ID"}</th><th>{"Text"}</th><th>{"Count"}</th><th>{"Gold reference"}</th><th>{"Left analysis"}</th><th>{"Right analysis"}</th></tr>
            </thead>
        <tbody>
        {for corpus.cases.iter().map(|c| {display_case(&c, &corpus)})}
            </tbody>
        </table>

    }
}

fn display_case(case: &common::Case, corpus: &common::Corpus) -> Html {
    html! {
            <tr style="border-bottom: 1px solid grey;">
            <td style="text-align:center">{&case.reference}</td>
            <td>{&case.text}</td>
            <td style="text-align:center">{&case.count}</td>
            <td>{display_annotations(&case.gold, &corpus)}</td>
            <td>{display_annotations(&case.left, &corpus)}</td>
            <td>{display_annotations(&case.right, &corpus)}</td>
            </tr>
    }
}

fn display_annotations(annots: &Vec<common::Annotation>, corpus: &common::Corpus) -> Html {
    html! {
        <table style="border-collapse:collapse">
        <tr><td>{for annots.iter().enumerate().map(|(i,annot)| {display_annotation(&annot, i, corpus)})}</td></tr>
        </table>
    }
}

fn hash_it<T:Hash>(t:T) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn display_annotation(annot: &common::Annotation, index: usize, corpus: &common::Corpus) -> Html {
    let color = hash_it(annot) % 360;
    let empty = "".to_string();
    let domain =corpus.intentMapping.val.get(&annot.intent).unwrap_or(&empty) ;

    html! {
        <table style={format!("border-collapse:separate; padding:0.2em; background-color:hsl({},35%,50%);",color)}>
            /*<thead>
            <tr>
            <th>{format!("Intent {}" ,index)}</th>
            <th>{"Properties"}</th>
            </tr>
            </thead>
            */
            <tbody>
            <tr ><td style={format!("background-color:hsl({},35%,50%);",color)}>
            <table style="border-collapse:collapse">
            <tr style={format!("background-color:hsl({},70%,80%);",(hash_it(&domain) % 360))}><td style="padding:0.25em;">{domain}</td></tr>
            <tr style={format!("background-color:hsl({},70%,80%);",(hash_it(&annot.intent) % 360))}><td style="padding:0.25em;">{&annot.intent}</td></tr>
            </table>
        </td>
            <td><table style="border-collapse:collapse">
            /*<thead>
            <tr><th>{"key"}</th><th>{"value"}</th></tr>
            </thead>*/
            <tbody>
            {for annot.values.iter().map(|kv| html!{<tr style={format!("background-color:hsl({},70%,80%);",(hash_it("".to_string() +&kv.0 + &kv.1) % 360))} ><td style="padding:0.25em;">{&kv.0}</td><td style="padding:0.25em;">{&kv.1}</td></tr>})}
        </tbody>
</table></td>
            </tr>
            </tbody>
            </table>
    }
}
