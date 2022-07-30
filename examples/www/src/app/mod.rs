use yew::prelude::*;

use body::Body;
use footer::Footer;
use header::Header;

mod body;
mod footer;
mod header;

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        App
    }

    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        false
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <>
                <Header title="PARSLEY Scheme" subtitle="a Scheme implementation in Rust" />
                <Body />
                <Footer />
            </>
        }
    }
}
