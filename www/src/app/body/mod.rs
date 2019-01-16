use yew::prelude::*;

mod terminal;

pub struct Body;

impl Component for Body {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Body
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}

impl Renderable<Body> for Body {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="PageBody",>
                <terminal::Terminal: />
            </div>
        }
    }
}
