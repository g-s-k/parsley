use yew::prelude::*;

pub struct Header {
    pub title: String,
    pub subtitle: String,
}

#[derive(Clone, Default, PartialEq)]
pub struct Props {
    pub title: String,
    pub subtitle: String,
}

impl Component for Header {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Header {
            title: props.title,
            subtitle: props.subtitle,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.title = props.title;
        self.subtitle = props.subtitle;
        true
    }
}

impl Renderable<Header> for Header {
    fn view(&self) -> Html<Self> {
        html! {
            <header class="Banner",>
                <h1 class="PageTitle",>
                    { &self.title }
                </h1>
                <p>
                    { &self.subtitle }
                </p>
                </header>
        }
    }
}
