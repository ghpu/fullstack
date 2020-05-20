use yew::prelude::*;

pub struct App {
    name: String,
    link:ComponentLink<Self>,
}

pub enum Msg {
    NoOp,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App { link, name: "toto".to_string()}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => {},
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>{"Hello world!"}</div>
        }
    }

    fn change(&mut self, _: <Self as yew::html::Component>::Properties) -> bool { false }

}
