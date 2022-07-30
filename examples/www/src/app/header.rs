use yew::prelude::*;

pub struct Header {
    pub title: String,
    pub subtitle: String,
}

#[derive(Clone, Default, PartialEq, Properties)]
pub struct Props {
    pub title: String,
    pub subtitle: String,
}

impl Component for Header {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Header {
            title: ctx.props().title.clone(),
            subtitle: ctx.props().subtitle.clone(),
        }
    }

    fn update(&mut self, _: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.title = ctx.props().title.clone();
        self.subtitle = ctx.props().subtitle.clone();
        true
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <header class="Banner">
                <h1 class="PageTitle">
                    { &self.title }
                </h1>
                <p>
                    { &self.subtitle }
                </p>
            </header>
        }
    }
}
