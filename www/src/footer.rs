use yew::prelude::*;

pub struct Footer;

impl Component for Footer {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Footer
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}

impl Renderable<Footer> for Footer {
    fn view(&self) -> Html<Self> {
        html! {
            <footer class="Footer",>
            { "The source for this site is available " }
            <a href="https://github.com/g-s-k/parsley",>{ "here" }</a>
            { "." }
            </footer>
        }
    }
}
