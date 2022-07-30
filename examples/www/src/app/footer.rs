use yew::prelude::*;

pub struct Footer;

impl Component for Footer {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Footer
    }

    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        false
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <footer class="Footer">
            { "The source for this site is available " }
            <a href="https://github.com/g-s-k/parsley">{ "here" }</a>
            { "." }
            </footer>
        }
    }
}
