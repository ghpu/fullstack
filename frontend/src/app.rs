use anyhow::Error;
use std::str::FromStr;
use common::{Annotation,Case, Corpus,IntentMapping, AnnotationComparison, compare, enum_str,AsStr, count};
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::{TimeoutService};
use yew::{Callback};
use std::time::Duration;
use std::hash::{Hash,Hasher};
use std::slice::Iter;
use std::cmp;
use unidecode;


pub struct App {
    link: ComponentLink<Self>,
    fetching: bool,
    ft: Option<FetchTask>,
    table: TableDisplay,
}

struct TableDisplay {
    current_index: usize,
    page_size: usize,
    corpus: common::Corpus,
    link_ref: ComponentLink<App>,
    sort_criterion: (TableField,SortDirection),
    filter: Option<String>,
    debounce_handle: yew::services::timeout::TimeoutTask,
    compare: CompareList,
    level: AnnotationComparison,
}

enum_str! {
    CompareList,
    (GoldVSLeft,"gold vs left"),
    (GoldVSRight,"gold vs right"),
    (RightVSLeft,"right vs left"),
}

enum_str!{
    TableField,
    (ID,"ID"),
    (Text,"Text"),
    (Count,"Count"),
    (Gold,"Gold"),
    (Left,"Left"),
    (Right,"Right"),
}

enum_str!{SortDirection,
    (Increasing," ↑"),
    (Decreasing," ↓"),
}



pub enum Msg {
    NoOp,
    FetchData,
    FetchReady(Result<Corpus, Error>),
    Ignore,
    UpdateCurrentIndex(usize),
    UpdatePageSize(ChangeData),
    UpdateSort(TableField),
    UpdateFilter(String),
    DebouncedExecution(String),
    UpdateCompare(ChangeData),
    UpdateMode(),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link: link.clone(),
            fetching: false,
            ft: None,
            table: TableDisplay{current_index: 0, page_size: 50, corpus: common::Corpus::empty(), link_ref:link.clone(), sort_criterion:(TableField::Text, SortDirection::Decreasing), filter: None, debounce_handle: TimeoutService::spawn(Duration::from_secs(1), link.clone().callback(|_| Msg::NoOp)), compare: CompareList::GoldVSLeft, level: AnnotationComparison::SameValues},
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link.callback(|_| Msg::FetchData).emit("");
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateCompare(cd) => {
                if let ChangeData::Select(se) = cd {
                    self.table.compare = CompareList::from_str(&se.value()).unwrap();
                }
            }
            Msg::UpdateMode() => {
                match self.table.level {
                    AnnotationComparison::SameValues => self.table.level = AnnotationComparison::SameProperties,
                    AnnotationComparison::SameProperties => self.table.level = AnnotationComparison::SameIntents,
                    AnnotationComparison::SameIntents=> self.table.level = AnnotationComparison::SameDomains,
                    AnnotationComparison::SameDomains => self.table.level = AnnotationComparison::Different,
                    AnnotationComparison::Different => self.table.level = AnnotationComparison::SameValues,
                }
            }

            Msg::UpdateFilter(filter_string) => { 
                self.table.debounce_handle = TimeoutService::spawn(Duration::from_millis(200), self.link.callback(move |_| Msg::DebouncedExecution(filter_string.clone()) ));
            }
            Msg::DebouncedExecution(filter_string) => {
                self.table.filter = if filter_string=="" {None} else {Some(filter_string)};
            }
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
                let task = FetchService::fetch(request, callback).unwrap();
                self.ft = Some(task);
            }
            Msg::FetchReady(response) => {
                self.fetching = false;
                self.table.corpus = response.unwrap_or(common::Corpus::empty()).clone();
                // add domain to all annotations
                for c in 0..self.table.corpus.cases.len() {
                    for a in 0..self.table.corpus.cases[c].gold.len() {
                        let mut ann = self.table.corpus.cases[c].gold[a].clone();
                        ann.domain = self.table.corpus.intentMapping.val.get(&ann.intent).unwrap_or(&"".to_string()).clone();
                        self.table.corpus.cases[c].gold[a] = ann;
                    }
                    for a in 0..self.table.corpus.cases[c].left.len() {
                        let mut ann = self.table.corpus.cases[c].left[a].clone();
                        ann.domain = self.table.corpus.intentMapping.val.get(&ann.intent).unwrap_or(&"".to_string()).clone();
                        self.table.corpus.cases[c].left[a] = ann;
                    }
                    for a in 0..self.table.corpus.cases[c].right.len() {
                        let mut ann = self.table.corpus.cases[c].right[a].clone();
                        ann.domain = self.table.corpus.intentMapping.val.get(&ann.intent).unwrap_or(&"".to_string()).clone();
                        self.table.corpus.cases[c].right[a] = ann;
                    }
                // Compute comparisons for all cases
                for c in 0..self.table.corpus.cases.len() {
                    self.table.corpus.cases[c].gold_vs_left = compare(&self.table.corpus.cases[c].gold, &self.table.corpus.cases[c].left);
                    self.table.corpus.cases[c].gold_vs_right = compare(&self.table.corpus.cases[c].gold, &self.table.corpus.cases[c].right);
                    self.table.corpus.cases[c].right_vs_left = compare(&self.table.corpus.cases[c].right, &self.table.corpus.cases[c].left);
                }


                }
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
            Msg::UpdateSort(f) => {
                let (c,d) = self.table.sort_criterion;
                if f == c {
                    if let SortDirection::Increasing = self.table.sort_criterion.1  {
                        self.table.sort_criterion.1 = SortDirection::Decreasing;
                    } else {
                    self.table.sort_criterion.1 = SortDirection::Increasing;
                    }
                }
                else {
                   self.table.sort_criterion = (f,SortDirection::Increasing);
                }
            }

        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div><h1>{ "Case analysis "}</h1>{  
                if self.fetching {html!{<p>{"Please wait, loading..."}</p>}} else {
                self.table.display()}
            }</div>
        }
    }

    fn change(&mut self, _: <Self as yew::html::Component>::Properties) -> bool {
        false
    }
}


fn sortFn(criterion: (TableField,SortDirection), a: &common::Case,b: &common::Case) -> std::cmp::Ordering {
    let (sort,direction) = criterion;
    let c = if let SortDirection::Increasing = direction {a} else {b};
    let d = if let SortDirection::Increasing =direction {b} else {a};
    match sort{
        TableField::ID => c.reference.partial_cmp(&d.reference).unwrap(),
        TableField::Text => unidecode::unidecode(&c.text).partial_cmp(&unidecode::unidecode(&d.text)).unwrap(),
        TableField::Count => c.count.partial_cmp(&d.count).unwrap(),
        TableField::Gold => c.gold.partial_cmp(&d.gold).unwrap(),
        TableField::Left => c.left.partial_cmp(&d.left).unwrap(),
        TableField::Right => c.right.partial_cmp(&d.right).unwrap()
    }
}

impl TableDisplay {
    fn display_header(&self, field:TableField) ->Html {
        let character = SortDirection::as_str(&self.sort_criterion.1);
        let name = TableField::as_str(&field);
        if field == self.sort_criterion.0 {
            html!{<button style="padding:0.3em; cursor: pointer" onclick=self.link_ref.callback(move |c| {Msg::UpdateSort(field)}) >{name}{character}</button>}
        } else {
            html!{
        <button style="padding:0.3em; cursor: pointer" onclick=self.link_ref.callback(move |c| {Msg::UpdateSort(field)})>{name}</button>
            }

        }
    }

    fn count_sentences(&self,what : &[common::Case]) -> String {
                format!("{} sentences ({} distinct)",
                what.iter().map(|c| {c.count}).sum::<usize>(),
                what.len()
 )
    }

    fn filter_fn(&self, case: &common::Case) -> bool {
        if let Some(f) = &self.filter { 
            if case.text.contains(f) { true } 
            else if case.gold.iter().any(|x| {x.intent.contains(f) || x.values.iter().any(|y| {y.0.contains(f) || y.1.contains(f)})}) {
                true
            }
            else if case.left.iter().any(|x| {x.intent.contains(f) || x.values.iter().any(|y| {y.0.contains(f) || y.1.contains(f)})}) {
                true
            }
            else if case.right.iter().any(|x| {x.intent.contains(f) || x.values.iter().any(|y| {y.0.contains(f) || y.1.contains(f)})}) {
                true
            }
            else {false}
        }
        else {
            true
        }
    }

    fn display(&self) -> Html {
        let mut current_cases = self.corpus.cases.to_vec();
        current_cases = current_cases.into_iter().filter(|x| self.filter_fn(x)).filter(|x| true  ).collect::<Vec<common::Case>>();
        current_cases.sort_by(move |a,b| {sortFn(self.sort_criterion,  a, b)});

        let current_case_page = if current_cases.len()>0 
        {&current_cases[self.current_index..std::cmp::min(current_cases.len(),self.current_index+self.page_size)]
        } else {&current_cases[..]};


        html! {
            <table style="border-collapse:collapse;">
                <thead>
                {self.display_filterbar(&current_cases)}
                {self.display_navbar(&current_cases)}
            <tr style="background-color:lightgrey;"><th>{self.display_header(TableField::ID)}</th><th>{self.display_header(TableField::Text)}</th><th>{self.display_header(TableField::Count)}</th><th>{self.display_header(TableField::Gold)}</th><th>{self.display_header(TableField::Left)}</th><th>{self.display_header(TableField::Right)}</th></tr>
                </thead>
                <tbody>
                {for current_case_page.iter().map(|c| {self.display_case(&c)})}
            </tbody>
                <tfoot>
                {self.display_navbar(&current_cases)}
            </tfoot>
                </table>

        }
    }

    fn display_filterbar(&self, cases: &[common::Case]) -> Html {
        html!{
            <tr style="background-color:lightgrey;"><th colspan="3">{"filter : "}<input type="text"  oninput=self.link_ref.callback(|x: InputData| Msg::UpdateFilter(x.value))/></th>
                <th colspan="2">
                <select onchange=self.link_ref.callback(|c| {Msg::UpdateCompare(c)})>
            { for CompareList::iterator().map( |v| {
                                                         html!{<option value=CompareList::as_str(v) selected= self.compare == *v  >{CompareList::as_str(v)}</option>}
                                                     })}
            </select>

                    //<button onclick=self.link_ref.callback(move |c| {Msg::UpdateCompare()})>{match self.compare { CompareList::GoldVSLeft => {"gold vs left"} , CompareList::GoldVSRight => {"gold vs right"}, CompareList::RightVSLeft => {"right vs left"}}}</button>
                    <button onclick=self.link_ref.callback(move |c| {Msg::UpdateMode()})>{AnnotationComparison::as_str(&self.level)}</button>

                </th>
                <th colspan="1">{self.count_sentences(&cases)}</th></tr>
        }
    }

    fn display_navbar(&self, cases: &[common::Case]) -> Html {
        let nb_pages = (cases.len()+self.page_size-1) / self.page_size;
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
                <th>
            <select value=self.page_size onchange=self.link_ref.callback(|c| {Msg::UpdatePageSize(c)})>
            { for [5,10,25,50,100].iter().map( |v| {
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
        let domain = &annot.domain; 

        html! {
            <table style={format!("border-collapse:separate; padding:0.2em; background-color:hsl({},35%,50%);",color)}>
                <tbody>
                <tr ><td style={format!("background-color:hsl({},35%,50%);",color)}>
                <table style={format!("border-collapse:collapse; padding:0.2em; background-color:hsl({},35%,50%);",color)} >
                <tr style={format!("text-align:center; background-color:hsl({},70%,80%);",(hash_it(&domain) % 360))}><td style="padding:0.25em;">{domain}</td></tr>
                <tr style={format!("text-align:center; background-color:hsl({},70%,80%);",(hash_it(&annot.intent) % 360))}><td style="padding:0.25em;">{&annot.intent}</td></tr>
                </table>
                </td>
                <td style={format!("border-collapse:collapse; padding:0.2em; background-color:hsl({},35%,50%);",color)}>
                <table style="border-collapse:collapse" >
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

