use yew::prelude::*;

mod terminal;

pub struct Body;

impl Component for Body {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Body
    }

    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        false
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <div class="PageBody">
                <terminal::Terminal />
            </div>
        }
    }
}
