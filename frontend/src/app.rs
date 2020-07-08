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
    ft: Option<FetchTask>,
    table: TableDisplay,
}

pub enum Msg {
    NoOp,
    FetchData,
    FetchReady(Result<Corpus, Error>),
    Ignore,
    UpdateCurrentIndex(usize),
    UpdatePageSize(ChangeData),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            console_service: ConsoleService::new(),
            fetch_service: FetchService::new(),
            link: link.clone(),
            fetching: false,
            ft: None,
            table: TableDisplay{current_index: 0, page_size: 50, corpus: common::Corpus::empty(), link_ref:link.clone()},
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
                self.table.corpus = response.unwrap_or(common::Corpus::empty()).clone();
            }
            Msg::Ignore => {
                self.fetching = false;
            }

            Msg::UpdatePageSize(cd) => {
                if let ChangeData::Select(se) = cd {
                    self.table.page_size = se.value().parse::<usize>().unwrap()
                }
            }

            Msg::UpdateCurrentIndex(ci) => {
                self.table.current_index=ci;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div><h1>{ "Hello world! "}</h1><p>{"Loading in progress: "}{self.fetching}</p><p>{  
                self.table.display()
            }</p></div>
        }
    }

    fn change(&mut self, _: <Self as yew::html::Component>::Properties) -> bool {
        false
    }
}

struct TableDisplay {
    current_index: usize,
    page_size: usize,
    corpus: common::Corpus,
    link_ref: ComponentLink<App>,
}

impl TableDisplay {
    fn display(&self) -> Html {
        let current_cases = if self.corpus.cases.len()>0 
        {&self.corpus.cases[self.current_index..std::cmp::min(self.corpus.cases.len(),self.current_index+self.page_size)]
        } else {&self.corpus.cases};
        html! {
            <table style="border-collapse:collapse;">
                <thead>
                <tr style="background-color:lightgrey;"><th colspan="6">{format!("{} sentences ({} distinct)",
                self.corpus.cases.iter().map(|c| {c.count}).sum::<usize>(),
                self.corpus.cases.len() )}</th></tr>
                {self.display_navbar()}
            <tr style="background-color:lightgrey;"><th>{"ID"}</th><th>{"Text"}</th><th>{"Count"}</th><th>{"Gold reference"}</th><th>{"Left analysis"}</th><th>{"Right analysis"}</th></tr>
                </thead>
                <tbody>
                {for current_cases.iter().map(|c| {self.display_case(&c)})}
            </tbody>
                <tfoot>
                {self.display_navbar()}
            </tfoot>
                </table>

        }
    }

    fn display_navbar(&self) -> Html {
        let nb_pages = (self.corpus.cases.len()+self.page_size-1) / self.page_size;
        let mut previous_page_list = vec![];
        let mut next_page_list = vec![];

        let page_size = self.page_size;

        let current_page = self.current_index / self.page_size + 1;


        if current_page as isize -100 > 0 { previous_page_list.push(current_page-100) };
        if current_page as isize -10 > 0 { previous_page_list.push(current_page-10) };
        if current_page as isize -5 > 0 { previous_page_list.push(current_page-5) };
        if current_page as isize -2 > 0 { previous_page_list.push(current_page-2) };
        if current_page as isize -1 > 0 { previous_page_list.push(current_page-1) };

        if current_page+1 <= nb_pages { next_page_list.push(current_page+1) };
        if current_page+2 <= nb_pages { next_page_list.push(current_page+2) };
        if current_page+5 <= nb_pages { next_page_list.push(current_page+5) };
        if current_page+10 <= nb_pages { next_page_list.push(current_page+10) };
        if current_page+100 <= nb_pages { next_page_list.push(current_page+100) };

        html! {<>
            <tr style="background-color:lightgrey;"><th colspan="5">
                <span style="display:inline-block; width:30%">{for previous_page_list.iter().map(|&i| {html!{
                                                         <button style="padding:0.3em; cursor: pointer" onclick=self.link_ref.callback(move |c| {Msg::UpdateCurrentIndex((i-1) * page_size)})
                                                             >{i}</button>
                                                     }})}</span>
            <span style="display: inline-block; width:20%; padding:0.3em;">{format!("page {}/{}", current_page, nb_pages)}</span>

<span style="display:inline-block; width:30%">{for next_page_list.iter().map(|&i| {html!{
                                                         <button style="padding:0.3em; cursor: pointer" onclick=self.link_ref.callback(move |c| {Msg::UpdateCurrentIndex((i-1) * page_size)})
                                                             >{i}</button>
                                                     }})}</span>
            </th>
                <th>{"number per page : "}
            <select value=self.page_size onchange=self.link_ref.callback(|c| {Msg::UpdatePageSize(c)})>
            { for [1,2,10,25,50,100].iter().map( |v| {
                                                         html!{<option value=*v selected= self.page_size == *v  >{*v}</option>}
                                                     })}
            </select>
                </th>
                </tr>
                </>
        }
    }


    fn display_case(&self, case: &common::Case) -> Html {
        html! {
            <tr style="border-bottom: 1px solid grey;">
                <td style="text-align:center">{&case.reference}</td>
                <td>{&case.text}</td>
                <td style="text-align:center">{&case.count}</td>
                <td>{self.display_annotations(&case.gold)}</td>
                <td>{self.display_annotations(&case.left)}</td>
                <td>{self.display_annotations(&case.right)}</td>
                </tr>
        }
    }

    fn display_annotations(&self, annots: &Vec<common::Annotation>) -> Html {
        html! {
            <table style="border-collapse:collapse">
            {for annots.iter().enumerate().map(|(i,annot)| html! {<tr><td> {self.display_annotation(&annot, i)}</td></tr> })}
            </table>
        }
    }


    fn display_annotation(&self, annot: &common::Annotation, index: usize) -> Html {
        let color = hash_it(annot) % 360;
        let empty = "".to_string();
        let domain =self.corpus.intentMapping.val.get(&annot.intent).unwrap_or(&empty) ;

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
                <table style={format!("border-collapse:collapse; padding:0.2em; background-color:hsl({},35%,50%);",color)} >
                <tr style={format!("text-align:center; background-color:hsl({},70%,80%);",(hash_it(&domain) % 360))}><td style="padding:0.25em;">{domain}</td></tr>
                <tr style={format!("text-align:center; background-color:hsl({},70%,80%);",(hash_it(&annot.intent) % 360))}><td style="padding:0.25em;">{&annot.intent}</td></tr>
                </table>
                </td>
                <td style={format!("border-collapse:collapse; padding:0.2em; background-color:hsl({},35%,50%);",color)}>
                <table style="border-collapse:collapse" >
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
}
fn hash_it<T:Hash>(t:T) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

