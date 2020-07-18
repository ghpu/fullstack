use anyhow::Error;
use std::str::FromStr;
use common::{Annotation,Case, Corpus, AnnotationComparison, compare, enum_str,AsStr, count};
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::TimeoutService;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use std::time::Duration;
use std::hash::{Hash,Hasher};
use std::slice::Iter;
use std::collections::{HashMap};
use unidecode;


pub struct App {
    link: ComponentLink<Self>,
    fetching: bool,
    //ft: Option<FetchTask>,
    global: GlobalDisplay,
    table: TableDisplay,
    graph: GraphDisplay,

    task: Option<ReaderTask>,
    corpus: Corpus
}

struct GlobalDisplay {
    gold: bool,
    left: bool,
    right:bool,
}

enum_str!{
    GlobalFilterMode,
    (None,"no filter"),
    (A,"a"),
    (B,"b"),
    (AORB,"a or b"),
}

#[derive(PartialEq,Eq,Clone)]
enum GlobalFilterTarget{
    Domain(String),
    Intent(String),
}

impl std::fmt::Display for GlobalFilterTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            GlobalFilterTarget::Domain(d) => write!(f, "domain : {}", d),
            GlobalFilterTarget::Intent(i) => write!(f, "intent : {}", i)
        }
    }
}

struct GraphDisplay {
    opened: bool,
    filter_mode: GlobalFilterMode,
    filter_target: GlobalFilterTarget,
}

struct TableDisplay {
    opened: bool,
    current_index: usize,
    page_size: usize,
    link_ref: ComponentLink<App>,
    sort_criterion: (TableField,SortDirection),
    filter: Option<String>,
    debounce_handle: yew::services::timeout::TimeoutTask,
    compare_mode:CompareList,
    compare_operator:Operator,
    compare_level:AnnotationComparison,
    compare_contains:GlobalFilterTarget,
}

enum_str! {
    CompareList,
    (GoldVSLeft,"gold vs left"),
    (GoldVSRight,"gold vs right"),
    (LeftVSRight,"left vs right"),
    (Gold,"gold"),
    (Left,"left"),
    (Right,"right"),
    (GoldOrLeft, "gold or left"),
    (GoldOrRight, "gold or right"),
    (LeftOrRight, "left or right"),
    (GoldOrLeftOrRight, "gold, left or right"),
}

enum_str! {
    Operator,
    (LTE,"<="),
    (GTE,">="),
    (EQ,"=="),
    (NEQ,"!="),
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
    //FetchData,
    FetchReady(Result<Corpus, Error>),
    //    Ignore,
    UpdateCurrentIndex(usize),
    UpdatePageSize(ChangeData),
    UpdateSort(TableField),
    UpdateFilter(String),
    DebouncedExecution(String),
    UpdateCompare(ChangeData),
    UpdateOperator(ChangeData),
    UpdateLevel(ChangeData),
    File(File),
    Loaded(String),
    ToggleTable,
    ToggleGraph,
    ToggleGold,
    ToggleLeft,
    ToggleRight,
    UpdateGraphFilterMode(ChangeData),
    UpdateGraphFilterTarget(ChangeData),
    UpdateTableFilterTarget(ChangeData),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link: link.clone(),
            fetching: false,
            //       ft: None,
            corpus: Corpus::empty(),
            global: GlobalDisplay{gold:true, left:true,right:true, },
            graph: GraphDisplay{opened: true, filter_mode: GlobalFilterMode::None, filter_target: GlobalFilterTarget::Domain("".to_string())},
            table: TableDisplay{opened: true, current_index: 0, page_size: 50, link_ref:link.clone(), sort_criterion:(TableField::ID, SortDirection::Increasing), filter: None, debounce_handle: TimeoutService::spawn(Duration::from_secs(1), link.clone().callback(|_| Msg::NoOp)), 
                compare_mode : CompareList::GoldVSLeft, compare_operator: Operator::LTE, compare_level: AnnotationComparison::SameValues, compare_contains: GlobalFilterTarget::Domain("".to_string())},
                task: None,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            //self.link.callback(|_| Msg::FetchData).emit("");
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateGraphFilterMode(cd) => {
                if let ChangeData::Select(se) = cd {
                    self.graph.filter_mode = GlobalFilterMode::from_str(&se.value()).unwrap();
                }
            }

            Msg::UpdateGraphFilterTarget(cd) => { 
                if let ChangeData::Select(se) = cd {
                    let s = &se.value();
                    if &s[0..2]=="d:" {
                        self.graph.filter_target = GlobalFilterTarget::Domain(s[2..].to_string());
                    }
                    if &s[0..2]=="i:" {
                        self.graph.filter_target = GlobalFilterTarget::Intent(s[2..].to_string());
                    }
                }
            }

            Msg::UpdateTableFilterTarget(cd) => { 
                if let ChangeData::Select(se) = cd {
                    let s = &se.value();
                    if &s[0..2]=="d:" {
                        self.table.compare_contains = GlobalFilterTarget::Domain(s[2..].to_string());
                    }
                    if &s[0..2]=="i:" {
                        self.table.compare_contains = GlobalFilterTarget::Intent(s[2..].to_string());
                    }
                }
            }

            Msg::ToggleGold => {
                self.global.gold= !self.global.gold;
            }
            Msg::ToggleLeft => {
                self.global.left= !self.global.left;
            }
            Msg::ToggleRight => {
                self.global.right= !self.global.right;
            }
            Msg::ToggleTable => {
                self.table.opened= !self.table.opened;
            }
            Msg::ToggleGraph => {
                self.graph.opened= !self.graph.opened;
            }
            Msg::Loaded(s) => {
                let data: Json<Result<Corpus, Error>> = Ok(s).into();
                let Json(dump) = data;
                self.link.callback(|x| Msg::FetchReady(x)).emit(dump);
            }
            Msg::File(file) => {
                let task = {
                    let callback = self.link.callback(|x:FileData| {Msg::Loaded(unsafe {String::from_utf8_unchecked(x.content)})});
                    ReaderService::new().read_file(file, callback).unwrap()
                };
                self.task=Some(task);
            }
            Msg::UpdateCompare(cd) => {
                if let ChangeData::Select(se) = cd {
                    self.table.compare_mode = CompareList::from_str(&se.value()).unwrap();
                }
            }
            Msg::UpdateOperator(cd) => {
                if let ChangeData::Select(se) = cd {
                    self.table.compare_operator = Operator::from_str(&se.value()).unwrap();
                }
            }
            Msg::UpdateLevel(cd) => {
                if let ChangeData::Select(se) = cd {
                    self.table.compare_level = AnnotationComparison::from_str(&se.value()).unwrap();
                }
            }
            Msg::UpdateFilter(filter_string) => { 
                self.table.debounce_handle = TimeoutService::spawn(Duration::from_millis(200), self.link.callback(move |_| Msg::DebouncedExecution(filter_string.clone()) ));
            }
            Msg::DebouncedExecution(filter_string) => {
                self.table.filter = if filter_string=="" {None} else {Some(filter_string)};
            }
            Msg::NoOp => {}
            /*
             * Msg::FetchData => {
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
             */
            Msg::FetchReady(response) => {
                self.fetching = false;
                self.corpus = response.unwrap_or(Corpus::empty()).clone();
                let filter_target = GlobalFilterTarget::Domain(self.corpus.intent_mapping.val.values().nth(0).unwrap().to_string());
                self.graph.filter_mode = GlobalFilterMode::None;
                self.graph.filter_target =filter_target.clone();
                self.table.compare_contains =filter_target;
                // add domain to all annotations
                for c in 0..self.corpus.cases.len() {
                    for a in 0..self.corpus.cases[c].gold.len() {
                        let mut ann = self.corpus.cases[c].gold[a].clone();
                        ann.domain = self.corpus.intent_mapping.val.get(&ann.intent).unwrap_or(&"".to_string()).clone();
                        self.corpus.cases[c].gold[a] = ann;
                    }
                    for a in 0..self.corpus.cases[c].left.len() {
                        let mut ann = self.corpus.cases[c].left[a].clone();
                        ann.domain = self.corpus.intent_mapping.val.get(&ann.intent).unwrap_or(&"".to_string()).clone();
                        self.corpus.cases[c].left[a] = ann;
                    }
                    for a in 0..self.corpus.cases[c].right.len() {
                        let mut ann = self.corpus.cases[c].right[a].clone();
                        ann.domain = self.corpus.intent_mapping.val.get(&ann.intent).unwrap_or(&"".to_string()).clone();
                        self.corpus.cases[c].right[a] = ann;
                    }
                }
                // Compute comparisons for all cases
                for c in 0..self.corpus.cases.len() {
                    self.corpus.cases[c].gold_vs_left = compare(&self.corpus.cases[c].gold, &self.corpus.cases[c].left);
                    self.corpus.cases[c].gold_vs_right = compare(&self.corpus.cases[c].gold, &self.corpus.cases[c].right);
                    self.corpus.cases[c].left_vs_right = compare(&self.corpus.cases[c].left, &self.corpus.cases[c].right);
                }


                self.table.opened = true;
            }
            /*
               Msg::Ignore => {
               self.fetching = false;
               }
               */

            Msg::UpdatePageSize(cd) => {
                if let ChangeData::Select(se) = cd {
                    self.table.page_size = se.value().parse::<usize>().unwrap()
                }
            }

            Msg::UpdateCurrentIndex(ci) => {
                self.table.current_index=ci;
            }
            Msg::UpdateSort(f) => {
                let (c,_) = self.table.sort_criterion;
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
            if self.fetching {html!{<p>{"Please wait, loading..."}</p>}} else {
                html!{<>
                    <div  style="background-color:lightgrey; padding:0">
                        <span><b>{"Case Analysis"}</b></span>
                        <input type="file" multiple=false onchange=self.link.callback(move |value|{
                            if let ChangeData::Files(file) = value {
                                Msg::File(file.get(0).unwrap())
                            } else { Msg::NoOp}
                        })/>

                    <button onclick=self.link.callback(|x| Msg::ToggleGraph) selected={self.graph.opened}>{if self.graph.opened {"✓ "} else {""}}{"📊"}</button>
                        <button onclick=self.link.callback(|x| Msg::ToggleTable) selected={self.table.opened}>{if self.table.opened {"✓ "} else {""}}{"📔"}</button>
                        <button onclick=self.link.callback(|x| Msg::ToggleGold)>{if self.global.gold {"✓ "} else {""}}{"Gold"}</button>
                        <button onclick=self.link.callback(|x| Msg::ToggleLeft)>{if self.global.left {"✓ "} else {""}}{"Left"}</button>
                        <button onclick=self.link.callback(|x| Msg::ToggleRight)>{if self.global.right {"✓ "} else {""}}{"Right"}</button>
                        </div>

                        {if self.graph.opened {self.graph.display(&self)} else {html!{}}}
                    {if self.table.opened {self.table.display(&self)} else {html!{}}}
                    </>
                }
            }
        }
    }

    fn change(&mut self, _: <Self as yew::html::Component>::Properties) -> bool {
        false
    }

}

impl App {

    fn graph_target_filter(&self, what: &Annotation) -> bool {
        match &self.graph.filter_target {
            GlobalFilterTarget::Domain(d) => what.domain==*d,
            GlobalFilterTarget::Intent(i) => what.intent==*i,
        }
    }

    fn graph_limit_filter(&self, what: &Case, mode: &CompareList ) -> bool {
        let (a,b) = match mode {
            CompareList::GoldVSLeft => (&what.gold, &what.left),
            CompareList::GoldVSRight => (&what.gold, &what.right),
            CompareList::LeftVSRight => (&what.left, &what.right),
            _ => panic!("not possible")
        };
        match self.graph.filter_mode  {
            GlobalFilterMode::None => true,
            GlobalFilterMode::A  =>  a.iter().any(|x| self.graph_target_filter(x) ),
            GlobalFilterMode::B => b.iter().any(|x| self.graph_target_filter(x) ),
            GlobalFilterMode::AORB => a.iter().any(|x| self.graph_target_filter(x)) || b.iter().any(|x| self.graph_target_filter(x)),
        }
    }

    fn table_target_filter(&self, what: &Annotation) -> bool {
        match &self.table.compare_contains {
            GlobalFilterTarget::Domain(d) => what.domain==*d,
            GlobalFilterTarget::Intent(i) => what.intent==*i,
        }
    }


    fn table_limit_filter(&self, what: &Case, mode: &CompareList ) -> bool {
        match mode {
            CompareList::Gold => what.gold.iter().any(|x| self.table_target_filter(x) ),
            CompareList::Left => what.left.iter().any(|x| self.table_target_filter(x) ),
            CompareList::Right => what.right.iter().any(|x| self.table_target_filter(x) ),
            CompareList::GoldOrLeft => what.gold.iter().any(|x| self.table_target_filter(x)) || what.left.iter().any(|x| self.table_target_filter(x)),
            CompareList::GoldOrRight=> what.gold.iter().any(|x| self.table_target_filter(x)) || what.right.iter().any(|x| self.table_target_filter(x)),
            CompareList::GoldOrLeftOrRight => what.gold.iter().any(|x| self.table_target_filter(x)) || what.left.iter().any(|x| self.table_target_filter(x)) || what.right.iter().any(|x| self.table_target_filter(x)),
            _ => panic!("not possible")
        }
    }


    fn display_graph_filter_infos(&self) -> Html {
        if let GlobalFilterMode::None = self.graph.filter_mode {
            html!{}
        } else {
            html!{<span>
                {"Limited to : "}{GlobalFilterMode::as_str(&self.graph.filter_mode)}{" containing "}{&self.graph.filter_target}
                </span>
            }
        }
    }

    fn display_graph_filter(&self) -> Html {
        let mut domains = self.corpus.intent_mapping.val.values().collect::<Vec<&String>>();
        domains.sort_unstable();
        domains.dedup();
        let mut intents = self.corpus.intent_mapping.val.keys().collect::<Vec<&String>>();
        intents.sort_unstable();
        intents.dedup();
        html!{<>
            <select onchange=self.link.callback(|c| {Msg::UpdateGraphFilterMode(c)})>
            { for GlobalFilterMode::iterator().map( |v| {
                                                            html!{<option value=GlobalFilterMode::as_str(v) selected= self.graph.filter_mode == *v  >{GlobalFilterMode::as_str(v)}</option>}
                                                        })}
            </select>
            {if let GlobalFilterMode::None = self.graph.filter_mode {html!{}} else {html!{
                                                                                             <select onchange=self.link.callback(|c| {Msg::UpdateGraphFilterTarget(c)})>
                                                                                             { for domains.iter().map( |d| {
                                                                                                                               html!{<option value="d:".to_string()+d selected= GlobalFilterTarget::Domain(d.to_string())  == self.graph.filter_target >{d}</option>}
                                                                                                                           })}
                                                                                             { for intents.iter().map( |i| {
                                                                                                                               html!{<option value="i:".to_string()+i selected= GlobalFilterTarget::Intent(i.to_string())  == self.graph.filter_target >{i}</option>}
                                                                                                                           })}


                                                                                             </select>}}}
            </>
        }

    }

}


fn sort_function(criterion: (TableField,SortDirection), a: &Case,b: &Case) -> std::cmp::Ordering {
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

impl GraphDisplay {
    fn display(&self, app: &App) -> Html {
        html! {<table id="graph">
            <thead><tr style="background-color:lightgrey;"><th colspan="3">{app.display_graph_filter()}</th></tr></thead>
                <tbody>
                <tr>
                {if app.global.gold && app.global.left 
                    {html!{<td>{self.display_pie(CompareList::GoldVSLeft, app)}</td>}} else {html!{<td/>}}}
            {if app.global.gold && app.global.right 
                {html!{<td>{self.display_pie(CompareList::GoldVSRight, app)}</td>}} else {html!{<td/>}}}
            {if app.global.left && app.global.right
                {html!{<td>{self.display_pie(CompareList::LeftVSRight, app)}</td>}} else {html!{<td/>}}}
            </tr>
            <tr>{if app.global.gold && app.global.left {html!{<td>{self.display_scatter(CompareList::GoldVSLeft, app)}</td>}} else {html!{<td/>}}}</tr>
            <tr>{if app.global.gold && app.global.left {html!{<td>{self.display_scatter(CompareList::GoldVSRight, app)}</td>}} else {html!{<td/>}}}</tr>
            <tr>{if app.global.gold && app.global.left {html!{<td>{self.display_scatter(CompareList::LeftVSRight, app)}</td>}} else {html!{<td/>}}}</tr>
                </tbody>
                </table>
        }
    }

    fn display_scatter(&self, mode: CompareList, app: &App) -> Html {
        html!{}
    }


    fn display_pie(&self, mode: CompareList, app: &App) -> Html {
        let pi: f32 = 3.14159265358979;
        let radius: f32 = 70.;
        let mut hm : HashMap<AnnotationComparison,usize> = HashMap::new();
        let mut current_cases = app.corpus.cases.to_vec();

        current_cases = match self.filter_mode {
            GlobalFilterMode::None => app.corpus.cases.to_vec(),
            _ => current_cases.into_iter().filter(|x| app.graph_limit_filter(x, &mode)).collect::<Vec<Case>>(),
        };

        for i in 0..current_cases.len() {
            let what =
                match mode {
                    CompareList::GoldVSLeft => &current_cases[i].gold_vs_left,
                    CompareList::GoldVSRight => &current_cases[i].gold_vs_right,
                    CompareList::LeftVSRight => &current_cases[i].left_vs_right,
                    _ => panic!("not possible"),
                };
            for e in what.iter() {
                let count = hm.entry(*e).or_insert(0);
                *count +=current_cases[i].count;
            }
        }

        let sum = hm.values().fold(0, |acc, x| acc + x);
        let mut offset = 0.;
        let mut pos = vec!();
        let colors=["#e31a1c","#feb24c","#ffffcc","#41ab5d","#005a32"];
        let mut color_index=0;
        for a in AnnotationComparison::iterator() {
            let length = *hm.get(a).unwrap_or(&0);
            pos.push(((2.*pi*radius*length as f32) / (sum as f32)
                      ,2.*pi*radius*(sum as f32 - length as f32) / (sum as f32)
                      ,2.*pi*radius*(0.25 - offset as f32)
                      , colors[color_index]
                     ));
            color_index +=1;
            offset += length as f32 / (sum as f32);
        };

        html!{<>
            <center><h3>{CompareList::as_str(&mode)}</h3></center>
                <center><h4>{app.display_graph_filter_infos()}</h4></center>
                <svg width="300" height="300" viewBox="0 0 300 300" fill="none" xmlns="http://www.w3.org/2000/svg">
                <defs>
                <linearGradient id="lights" x1="1" x2="0" y1="1" y2="0">
                <stop offset="0%" stop-color="rgba(254,252,234,0.2)"/>
                <stop offset="100%" stop-color="rgba(241,218,54,0.3)"/>
                </linearGradient>
                </defs>
                <circle cx="152" cy="152" r={radius *5./4.} fill="#444"></circle>
                <circle cx="152" cy="152" r={radius *3./4.} fill="#fff"></circle>
                <text x="150" y="150" style="fill:black; text-anchor: middle; dominant-baseline: middle;" font-size="24">{format!("{:.1}", *hm.get(&AnnotationComparison::SameValues).unwrap_or(&0) as f32 / (sum as f32) * 100.)}</text>
                {for pos.iter().map(|p| html!{
                                                 <circle cx="150" cy="150" r={radius} fill="transparent" stroke={format!("{}",p.3)} stroke-width={format!("{}",0.5*radius)} stroke-dasharray={format!("{} {}", p.0,p.1)} stroke-dashoffset={format!("{}",p.2)}></circle>

                                             })}
            <circle cx="150" cy="150" r={radius} stroke="url(#lights)" stroke-width={format!("{}",0.5*radius)} ></circle>

                </svg>
                <table>
                {for AnnotationComparison::iterator().map(|v| html!{
                                                                       <tr>
                                                                           <td>{AnnotationComparison::as_str(v)}</td>
                                                                           <td>{hm.get(v).unwrap_or(&0)}</td>
                                                                           <td>{format!("{:.2}%", *hm.get(v).unwrap_or(&0) as f32 / (sum as f32) * 100.)}</td>
                                                                           </tr>})}
            <tfoot><tr><td>{"total"}</td><td>{sum}</td><td></td></tr></tfoot>
                </table>
                </>
        }
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

    fn count_sentences(&self,what : &[Case]) -> String {
        format!("{} sentences ({} distinct)",
        what.iter().map(|c| {c.count}).sum::<usize>(),
        what.len()
        )
    }

    fn filter_fn(&self, case: &Case) -> bool {
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

    fn filter_comparison(&self, c: &Case, app: &App) -> bool {
        match &self.compare_mode {
            CompareList::GoldVSLeft | CompareList::GoldVSRight | CompareList::LeftVSRight => {
                let d = match &self.compare_mode {
                    CompareList::GoldVSLeft => &c.gold_vs_left, 
                    CompareList::GoldVSRight => &c.gold_vs_right, 
                    CompareList::LeftVSRight => &c.left_vs_right,
                    _ => panic!("not possible")
                };

                match &self.compare_operator {
                    Operator::LTE => d.iter().any( |x| x <= &self.compare_level),
                    Operator::GTE => d.iter().any( |x| x >= &self.compare_level),
                    Operator::EQ => d.iter().any( |x| x== &self.compare_level),
                    Operator::NEQ => d.iter().any( |x| x != &self.compare_level),
                }

            },

            _ => app.table_limit_filter(c, &self.compare_mode), // Gold, Right, Left, GoldOrLeft...

        }



    }

    fn display(&self, app: &App) -> Html {
        let mut current_cases = app.corpus.cases.to_vec();
        // table filter
        current_cases = current_cases.into_iter().filter(|x| self.filter_fn(x)).filter(|c| self.filter_comparison(c,app)).collect::<Vec<Case>>();


        current_cases.sort_by(move |a,b| {sort_function(self.sort_criterion,  a, b)});

        let current_case_page = if current_cases.len()>0 
        {&current_cases[self.current_index..std::cmp::min(current_cases.len(),self.current_index+self.page_size)]
        } else {&current_cases[..]};


        html! {
            <table id="table" style="border-collapse:collapse;">
                <thead>
                {self.display_filterbar(&current_cases, &app)}
            <tr style="background-color:lightgrey;">
                <th>{self.display_header(TableField::ID)}</th>
                <th>{self.display_header(TableField::Text)}</th>
                <th>{self.display_header(TableField::Count)}</th>
                {if app.global.gold {html!{
                                              <th>{self.display_header(TableField::Gold)}</th>}} else {html!{<th/>}}}
            {if app.global.left {html!{
                                          <th>{self.display_header(TableField::Left)}</th>}} else {html!{<th/>}}}
            {if app.global.right {html!{
                                           <th>{self.display_header(TableField::Right)}</th>}} else {html!{<th/>}}}
            </tr>
                </thead>
                <tbody>
                {for current_case_page.iter().map(|c| {self.display_case(&c, app)})}
            </tbody>
                <tfoot>
                {self.display_navbar(&current_cases)}
            </tfoot>
                </table>

        }
    }

    fn display_filterbar(&self, cases: &[Case], app: &App) -> Html {
        html!{
            <>
                <tr style="background-color:lightgrey;"><th colspan="6"><span>{self.count_sentences(&cases)}</span></th></tr>
                <tr style="background-color:lightgrey;">
                <th colspan="6"><span>{"text filter : "}</span><input type="text"  oninput=self.link_ref.callback(|x: InputData| Msg::UpdateFilter(x.value))/>
                <span>{ "comparison mode : "}</span>
                <select onchange=self.link_ref.callback(|c| {Msg::UpdateCompare(c)})>
                {if app.global.gold && app.global.left 
                    {html!{<option value=CompareList::as_str(&CompareList::GoldVSLeft) selected = self.compare_mode == CompareList::GoldVSLeft  >{CompareList::as_str(&CompareList::GoldVSLeft)}</option>}}
                    else {html!{}}
                }
            {if app.global.gold && app.global.right 
                {html!{<option value=CompareList::as_str(&CompareList::GoldVSRight) selected =  self.compare_mode == CompareList::GoldVSRight > {CompareList::as_str(&CompareList::GoldVSRight)}</option>}}
                else {html!{}}
            }
            {if app.global.left && app.global.right
                {html!{<option value=CompareList::as_str(&CompareList::LeftVSRight) selected = self.compare_mode == CompareList::LeftVSRight >{CompareList::as_str(&CompareList::LeftVSRight)}</option>}}
                else {html!{}}
            }
            {if app.global.gold
                {html!{<option value=CompareList::as_str(&CompareList::Gold) selected = self.compare_mode == CompareList::Gold>{CompareList::as_str(&CompareList::Gold)}</option>}}
                else {html!{}}
            }
            {if app.global.left
                {html!{<option value=CompareList::as_str(&CompareList::Left) selected = self.compare_mode == CompareList::Left>{CompareList::as_str(&CompareList::Left)}</option>}}
                else {html!{}}
            }
            {if app.global.right
                {html!{<option value=CompareList::as_str(&CompareList::Right) selected = self.compare_mode == CompareList::Right>{CompareList::as_str(&CompareList::Right)}</option>}}
                else {html!{}}
            }

            {if app.global.gold && app.global.left
                {html!{<option value=CompareList::as_str(&CompareList::GoldOrLeft) selected = self.compare_mode == CompareList::GoldOrLeft>{CompareList::as_str(&CompareList::GoldOrLeft)}</option>}}
                else {html!{}}
            }

            {if app.global.gold && app.global.right
                {html!{<option value=CompareList::as_str(&CompareList::GoldOrRight) selected = self.compare_mode == CompareList::GoldOrRight>{CompareList::as_str(&CompareList::GoldOrRight)}</option>}}
                else {html!{}}
            }

            {if app.global.left && app.global.right
                {html!{<option value=CompareList::as_str(&CompareList::LeftOrRight) selected = self.compare_mode == CompareList::LeftOrRight>{CompareList::as_str(&CompareList::LeftOrRight)}</option>}}
                else {html!{}}
            }

            {if app.global.gold && app.global.left && app.global.right
                {html!{<option value=CompareList::as_str(&CompareList::GoldOrLeftOrRight) selected = self.compare_mode == CompareList::GoldOrLeftOrRight>{CompareList::as_str(&CompareList::GoldOrLeftOrRight)}</option>}}
                else {html!{}}
            }

            </select>

            { match self.compare_mode {
                                          CompareList::GoldVSLeft | CompareList::GoldVSRight | CompareList:: LeftVSRight =>
                                              html!{
                                                  <>
                                                      <select onchange=self.link_ref.callback(|c| {Msg::UpdateOperator(c)})>
                                                      { for Operator::iterator().map( |v| {
                                                                                              html!{<option value=Operator::as_str(v) selected= self.compare_operator == *v  >{Operator::as_str(v)}</option>}
                                                                                          })}
                                                  </select>

                                                      <select onchange=self.link_ref.callback(|c| {Msg::UpdateLevel(c)})>
                                                      { for AnnotationComparison::iterator().map( |v| {
                                                                                                          html!{<option value=AnnotationComparison::as_str(v) selected= self.compare_level == *v  >{AnnotationComparison::as_str(v)}</option>}
                                                                                                      })}
                                                  </select></>

                                              },

                                          _ => {
                                              let mut domains = app.corpus.intent_mapping.val.values().collect::<Vec<&String>>();
                                              domains.sort_unstable();
                                              domains.dedup();
                                              let mut intents = app.corpus.intent_mapping.val.keys().collect::<Vec<&String>>();
                                              intents.sort_unstable();
                                              intents.dedup();
                                              html!{<>
                                                  <span>{"contains : "}</span>
                                                      <select onchange=self.link_ref.callback(|c| {Msg::UpdateTableFilterTarget(c)})>
                                                      { for domains.iter().map( |d| {
                                                                                        html!{<option value="d:".to_string()+d selected= GlobalFilterTarget::Domain(d.to_string())  == self.compare_contains >{d}</option>}
                                                                                    })}
                                                  { for intents.iter().map( |i| {
                                                                                    html!{<option value="i:".to_string()+i selected= GlobalFilterTarget::Intent(i.to_string())  == self.compare_contains >{i}</option>}
                                                                                })}


                                                  </select></>
                                              }

                                          }



                                      }}

            </th>
                </tr>
                </>
        }
    }

    fn display_navbar(&self, cases: &[Case]) -> Html {
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


    fn display_case(&self, case: &Case, app: &App) -> Html {
        html! {
            <tr style="border-bottom: 1px solid grey;">
                <td style="text-align:center">{&case.reference}</td>
                <td>{&case.text}</td>
                <td style="text-align:center">{&case.count}</td>
                {if {app.global.gold}
                    {html!{<td>{self.display_annotations(&case.gold)}</td>}} else {html!{<td/>}}}
            {if {app.global.left}
                {html!{<td>{self.display_annotations(&case.left)}</td>}} else {html!{<td/>}}}
            {if {app.global.right}
                {html!{<td>{self.display_annotations(&case.right)}</td>}} else {html!{<td/>}}}
            </tr>
        }
    }

    fn display_annotations(&self, annots: &Vec<Annotation>) -> Html {
        html! {
            <table style="border-collapse:collapse">
            {for annots.iter().map(|annot| html! {<tr><td> {self.display_annotation(&annot)}</td></tr> })}
            </table>
        }
    }


    fn display_annotation(&self, annot: &Annotation) -> Html {
        let color = hash_it(annot) % 360;
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

